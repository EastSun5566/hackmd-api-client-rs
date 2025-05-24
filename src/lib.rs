pub mod error;
pub mod types;

pub use error::{ApiError, Result};
pub use types::*;

use crate::error::{
    HttpResponseError, InternalServerError, MissingRequiredArgument, TooManyRequestsError,
};
use reqwest::{header, Client as HttpClient, Response, StatusCode, Url};
use serde_json::Value;
use std::{future, time};

const DEFAULT_BASE_URL: &str = "https://api.hackmd.io/v1";

#[derive(Clone)]
pub struct ApiClientOptions {
    pub wrap_response_errors: bool,
    pub timeout: Option<time::Duration>,
    pub retry_options: Option<RetryOptions>,
}

impl Default for ApiClientOptions {
    fn default() -> Self {
        Self {
            wrap_response_errors: true,
            timeout: Some(time::Duration::from_secs(30)),
            retry_options: Some(RetryOptions {
                max_retries: 3,
                base_delay: time::Duration::from_millis(100),
            }),
        }
    }
}

#[derive(Clone)]
pub struct RetryOptions {
    pub max_retries: u32,
    pub base_delay: time::Duration,
}

pub struct ApiClient {
    http_client: HttpClient,
    base_url: Url,
    options: ApiClientOptions,
}

impl ApiClient {
    pub fn new(access_token: &str) -> Result<Self> {
        Self::with_options(access_token, None, None)
    }

    pub fn with_base_url(access_token: &str, base_url: &str) -> Result<Self> {
        Self::with_options(access_token, Some(base_url), None)
    }

    pub fn with_options(
        access_token: &str,
        base_url: Option<&str>,
        options: Option<ApiClientOptions>,
    ) -> Result<Self> {
        if access_token.is_empty() {
            return Err(ApiError::MissingRequiredArgument(MissingRequiredArgument {
                message: "Missing access token when creating HackMD client".to_string(),
            }));
        }

        let options = options.unwrap_or_default();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let mut client_builder = HttpClient::builder().default_headers(headers);

        if let Some(timeout) = options.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let http_client = client_builder.build()?;
        let base_url = Url::parse(base_url.unwrap_or(DEFAULT_BASE_URL))?;

        Ok(Self {
            http_client,
            base_url,
            options,
        })
    }

    async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();

        if !self.options.wrap_response_errors {
            return if status.is_success() {
                Ok(response.json().await?)
            } else {
                Err(ApiError::Reqwest(response.error_for_status().unwrap_err()))
            };
        }

        if status.is_success() {
            return Ok(response.json().await?);
        }

        let status_text = status.canonical_reason().unwrap_or("Unknown").to_string();

        match status {
            StatusCode::TOO_MANY_REQUESTS => {
                let user_limit = response
                    .headers()
                    .get("x-ratelimit-userlimit")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);

                let user_remaining = response
                    .headers()
                    .get("x-ratelimit-userremaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0);

                let reset_after = response
                    .headers()
                    .get("x-ratelimit-userreset")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse().ok());

                Err(ApiError::TooManyRequests(TooManyRequestsError {
                    message: format!("Too many requests ({} {})", status.as_u16(), status_text),
                    code: status.as_u16(),
                    status_text,
                    user_limit,
                    user_remaining,
                    reset_after,
                }))
            }
            _ if status.is_server_error() => Err(ApiError::InternalServer(InternalServerError {
                message: format!(
                    "HackMD internal error ({} {})",
                    status.as_u16(),
                    status_text
                ),
                code: status.as_u16(),
                status_text,
            })),
            _ => Err(ApiError::HttpResponse(HttpResponseError {
                message: format!(
                    "Received an error response ({} {}) from HackMD",
                    status.as_u16(),
                    status_text
                ),
                code: status.as_u16(),
                status_text,
            })),
        }
    }

    async fn retry_request<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: future::Future<Output = Result<T>>,
    {
        let retry_options = match &self.options.retry_options {
            Some(config) => config,
            None => return operation().await,
        };

        let mut last_error = None;
        for attempt in 0..=retry_options.max_retries {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    if attempt < retry_options.max_retries && self.is_retryable_error(&err) {
                        let delay = self.exponential_backoff(attempt, retry_options.base_delay);
                        tokio::time::sleep(delay).await;
                        last_error = Some(err);
                    } else {
                        return Err(err);
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    fn is_retryable_error(&self, error: &ApiError) -> bool {
        match error {
            ApiError::TooManyRequests(err) => err.user_remaining > 0,
            ApiError::InternalServer(_) => true,
            ApiError::Reqwest(req_err) => {
                req_err.is_timeout() || req_err.is_connect() || req_err.is_request()
            }
            _ => false,
        }
    }

    fn exponential_backoff(&self, retries: u32, base_delay: time::Duration) -> time::Duration {
        let multiplier = 2_u64.pow(retries);
        time::Duration::from_millis(base_delay.as_millis() as u64 * multiplier)
    }

    // User API methods
    pub async fn get_me(&self) -> Result<User> {
        self.retry_request(|| async {
            let url = self.base_url.join("me")?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_history(&self) -> Result<Vec<Note>> {
        self.retry_request(|| async {
            let url = self.base_url.join("history")?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_note_list(&self) -> Result<Vec<Note>> {
        self.retry_request(|| async {
            let url = self.base_url.join("notes")?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_note(&self, note_id: &str) -> Result<SingleNote> {
        self.retry_request(|| async {
            let url = self.base_url.join(&format!("notes/{}", note_id))?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn create_note(&self, payload: &CreateNoteOptions) -> Result<SingleNote> {
        self.retry_request(|| async {
            let url = self.base_url.join("notes")?;
            let response = self.http_client.post(url).json(payload).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn update_note_content(&self, note_id: &str, content: &str) -> Result<SingleNote> {
        let payload = UpdateNoteOptions {
            content: Some(content.to_string()),
            read_permission: None,
            write_permission: None,
            permalink: None,
        };
        self.update_note(note_id, &payload).await
    }

    pub async fn update_note(
        &self,
        note_id: &str,
        payload: &UpdateNoteOptions,
    ) -> Result<SingleNote> {
        self.retry_request(|| async {
            let url = self.base_url.join(&format!("notes/{}", note_id))?;
            let response = self.http_client.patch(url).json(payload).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn delete_note(&self, note_id: &str) -> Result<()> {
        self.retry_request(|| async {
            let url = self.base_url.join(&format!("notes/{}", note_id))?;
            let response = self.http_client.delete(url).send().await?;
            let _: Value = self.handle_response(response).await?;
            Ok(())
        })
        .await
    }

    // Team API methods
    pub async fn get_teams(&self) -> Result<Vec<Team>> {
        self.retry_request(|| async {
            let url = self.base_url.join("teams")?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_team_notes(&self, team_path: &str) -> Result<Vec<Note>> {
        self.retry_request(|| async {
            let url = self.base_url.join(&format!("teams/{}/notes", team_path))?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn create_team_note(
        &self,
        team_path: &str,
        payload: &CreateNoteOptions,
    ) -> Result<SingleNote> {
        self.retry_request(|| async {
            let url = self.base_url.join(&format!("teams/{}/notes", team_path))?;
            let response = self.http_client.post(url).json(payload).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn update_team_note_content(
        &self,
        team_path: &str,
        note_id: &str,
        content: &str,
    ) -> Result<()> {
        let payload = UpdateNoteOptions {
            content: Some(content.to_string()),
            read_permission: None,
            write_permission: None,
            permalink: None,
        };
        self.update_team_note(team_path, note_id, &payload).await
    }

    pub async fn update_team_note(
        &self,
        team_path: &str,
        note_id: &str,
        payload: &UpdateNoteOptions,
    ) -> Result<()> {
        self.retry_request(|| async {
            let url = self
                .base_url
                .join(&format!("teams/{}/notes/{}", team_path, note_id))?;
            let response = self.http_client.patch(url).json(payload).send().await?;
            let _: Value = self.handle_response(response).await?;
            Ok(())
        })
        .await
    }

    pub async fn delete_team_note(&self, team_path: &str, note_id: &str) -> Result<()> {
        self.retry_request(|| async {
            let url = self
                .base_url
                .join(&format!("teams/{}/notes/{}", team_path, note_id))?;
            let response = self.http_client.delete(url).send().await?;
            let _: Value = self.handle_response(response).await?;
            Ok(())
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_client_creation() {
        let client = ApiClient::new("test_token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_client_creation_empty_token() {
        let client = ApiClient::new("");
        assert!(client.is_err());

        if let Err(ApiError::MissingRequiredArgument(err)) = client {
            assert!(err.message.contains("Missing access token"));
        } else {
            panic!("Expected MissingRequiredArgument error");
        }
    }

    #[test]
    fn test_api_client_with_base_url() {
        let client = ApiClient::with_base_url("test_token", "https://api.example.com/v1");
        assert!(client.is_ok());
    }

    #[test]
    fn test_api_client_with_options() {
        let options = ApiClientOptions {
            wrap_response_errors: false,
            timeout: Some(time::Duration::from_secs(10)),
            retry_options: None,
        };

        let client = ApiClient::with_options("test_token", None, Some(options));
        assert!(client.is_ok());
    }

    #[test]
    fn test_create_note_options_serialization() {
        let options = CreateNoteOptions {
            title: Some("Test Note".to_string()),
            content: Some("# Test Content".to_string()),
            read_permission: Some(NotePermissionRole::Owner),
            write_permission: Some(NotePermissionRole::SignedIn),
            comment_permission: Some(CommentPermissionType::Owners),
            permalink: None,
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("Test Note"));
        assert!(json.contains("Test Content"));
    }

    #[test]
    fn test_update_note_options_serialization() {
        let options = UpdateNoteOptions {
            content: Some("Updated content".to_string()),
            read_permission: None,
            write_permission: Some(NotePermissionRole::Guest),
            permalink: Some("custom-permalink".to_string()),
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("Updated content"));
        assert!(json.contains("guest"));
        assert!(json.contains("custom-permalink"));
        // Should not contain null values for None fields
        assert!(!json.contains("readPermission"));
    }
}
