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

const DEFAULT_BASE_URL: &str = "https://api.hackmd.io/v1/";

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
            retry_options: Some(RetryOptions::default()),
        }
    }
}

#[derive(Clone)]
pub struct RetryOptions {
    pub max_retries: u32,
    pub base_delay: time::Duration,
}

impl Default for RetryOptions {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: time::Duration::from_millis(100),
        }
    }
}

pub struct ApiClient {
    http_client: HttpClient,
    base_url: Url,
    options: ApiClientOptions,
}

impl ApiClient {
    fn missing_required_argument(message: impl Into<String>) -> ApiError {
        ApiError::MissingRequiredArgument(MissingRequiredArgument {
            message: message.into(),
        })
    }

    fn require_non_empty(value_name: &str, value: &str) -> Result<()> {
        if value.trim().is_empty() {
            return Err(Self::missing_required_argument(format!(
                "Missing {value_name} when calling HackMD API"
            )));
        }

        Ok(())
    }

    fn normalized_base_url(base_url: &str) -> String {
        if base_url.ends_with('/') {
            base_url.to_string()
        } else {
            format!("{base_url}/")
        }
    }

    fn note_url(&self, note_id: &str) -> Result<Url> {
        Self::require_non_empty("note_id", note_id)?;
        Ok(self.base_url.join(&format!("notes/{note_id}"))?)
    }

    fn note_image_url(&self, note_id: &str) -> Result<Url> {
        Self::require_non_empty("note_id", note_id)?;
        Ok(self.base_url.join(&format!("notes/{note_id}/images"))?)
    }

    fn folders_url(&self) -> Result<Url> {
        Ok(self.base_url.join("folders")?)
    }

    fn folder_order_url(&self) -> Result<Url> {
        Ok(self.base_url.join("folders/folder-order")?)
    }

    fn folder_url(&self, folder_id: &str) -> Result<Url> {
        Self::require_non_empty("folder_id", folder_id)?;
        Ok(self.base_url.join(&format!("folders/{folder_id}"))?)
    }

    fn team_notes_url(&self, team_path: &str) -> Result<Url> {
        Self::require_non_empty("team_path", team_path)?;
        Ok(self.base_url.join(&format!("teams/{team_path}/notes"))?)
    }

    fn team_note_url(&self, team_path: &str, note_id: &str) -> Result<Url> {
        Self::require_non_empty("team_path", team_path)?;
        Self::require_non_empty("note_id", note_id)?;
        Ok(self
            .base_url
            .join(&format!("teams/{team_path}/notes/{note_id}"))?)
    }

    fn team_folders_url(&self, team_path: &str) -> Result<Url> {
        Self::require_non_empty("team_path", team_path)?;
        Ok(self.base_url.join(&format!("teams/{team_path}/folders"))?)
    }

    fn team_folder_order_url(&self, team_path: &str) -> Result<Url> {
        Self::require_non_empty("team_path", team_path)?;
        Ok(self
            .base_url
            .join(&format!("teams/{team_path}/folders/folder-order"))?)
    }

    fn team_folder_url(&self, team_path: &str, folder_id: &str) -> Result<Url> {
        Self::require_non_empty("team_path", team_path)?;
        Self::require_non_empty("folder_id", folder_id)?;
        Ok(self
            .base_url
            .join(&format!("teams/{team_path}/folders/{folder_id}"))?)
    }

    fn is_success_status(status: StatusCode) -> bool {
        status.is_success()
    }

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
        if access_token.trim().is_empty() {
            return Err(Self::missing_required_argument(
                "Missing access token when creating HackMD client",
            ));
        }

        let options = options.unwrap_or_default();

        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );

        let mut client_builder = HttpClient::builder().default_headers(headers);

        if let Some(timeout) = options.timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let http_client = client_builder.build()?;
        let base_url = Url::parse(&Self::normalized_base_url(
            base_url.unwrap_or(DEFAULT_BASE_URL),
        ))?;

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

    async fn handle_empty_response(&self, response: Response) -> Result<()> {
        if Self::is_success_status(response.status()) {
            return Ok(());
        }

        self.handle_response::<Value>(response).await.map(|_| ())
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
            ApiError::TooManyRequests(_) => true,
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

    pub async fn get_history(&self, limit: Option<u32>) -> Result<Vec<Note>> {
        self.retry_request(|| async {
            let mut url = self.base_url.join("history")?;
            if let Some(limit_val) = limit {
                url.query_pairs_mut()
                    .append_pair("limit", &limit_val.to_string());
            }
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
            let url = self.note_url(note_id)?;
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

    pub async fn update_note_content(&self, note_id: &str, content: &str) -> Result<()> {
        let payload = UpdateNoteOptions {
            content: Some(content.to_string()),
            ..Default::default()
        };
        self.update_note(note_id, &payload).await
    }

    pub async fn update_note(&self, note_id: &str, payload: &UpdateNoteOptions) -> Result<()> {
        self.retry_request(|| async {
            let url = self.note_url(note_id)?;
            let response = self.http_client.patch(url).json(payload).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn delete_note(&self, note_id: &str) -> Result<()> {
        self.retry_request(|| async {
            let url = self.note_url(note_id)?;
            let response = self.http_client.delete(url).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn upload_note_image(
        &self,
        note_id: &str,
        image_bytes: bytes::Bytes,
        file_name: &str,
        mime_type: &str,
    ) -> Result<NoteImageUploadResponse> {
        self.retry_request(|| async {
            let url = self.note_image_url(note_id)?;
            let part = reqwest::multipart::Part::stream(image_bytes.clone())
                .file_name(file_name.to_string())
                .mime_str(mime_type)?;
            let form = reqwest::multipart::Form::new().part("image", part);
            let response = self.http_client.post(url).multipart(form).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_folders(&self) -> Result<Vec<Folder>> {
        self.retry_request(|| async {
            let url = self.folders_url()?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn create_folder(&self, payload: &CreateFolderOptions) -> Result<Folder> {
        self.retry_request(|| async {
            let url = self.folders_url()?;
            let response = self.http_client.post(url).json(payload).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_folder(&self, folder_id: &str) -> Result<Folder> {
        self.retry_request(|| async {
            let url = self.folder_url(folder_id)?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn update_folder(
        &self,
        folder_id: &str,
        payload: &UpdateFolderOptions,
    ) -> Result<()> {
        self.retry_request(|| async {
            let url = self.folder_url(folder_id)?;
            let response = self.http_client.patch(url).json(payload).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn delete_folder(&self, folder_id: &str) -> Result<()> {
        self.retry_request(|| async {
            let url = self.folder_url(folder_id)?;
            let response = self.http_client.delete(url).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn get_folder_order(&self) -> Result<FolderOrder> {
        self.retry_request(|| async {
            let url = self.folder_order_url()?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn update_folder_order(&self, payload: &UpdateFolderOrderOptions) -> Result<()> {
        self.retry_request(|| async {
            let url = self.folder_order_url()?;
            let response = self.http_client.put(url).json(payload).send().await?;
            self.handle_empty_response(response).await
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
            let url = self.team_notes_url(team_path)?;
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
            let url = self.team_notes_url(team_path)?;
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
            ..Default::default()
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
            let url = self.team_note_url(team_path, note_id)?;
            let response = self.http_client.patch(url).json(payload).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn delete_team_note(&self, team_path: &str, note_id: &str) -> Result<()> {
        self.retry_request(|| async {
            let url = self.team_note_url(team_path, note_id)?;
            let response = self.http_client.delete(url).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn get_team_folders(&self, team_path: &str) -> Result<Vec<Folder>> {
        self.retry_request(|| async {
            let url = self.team_folders_url(team_path)?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn create_team_folder(
        &self,
        team_path: &str,
        payload: &CreateFolderOptions,
    ) -> Result<Folder> {
        self.retry_request(|| async {
            let url = self.team_folders_url(team_path)?;
            let response = self.http_client.post(url).json(payload).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn get_team_folder(&self, team_path: &str, folder_id: &str) -> Result<Folder> {
        self.retry_request(|| async {
            let url = self.team_folder_url(team_path, folder_id)?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn update_team_folder(
        &self,
        team_path: &str,
        folder_id: &str,
        payload: &UpdateFolderOptions,
    ) -> Result<()> {
        self.retry_request(|| async {
            let url = self.team_folder_url(team_path, folder_id)?;
            let response = self.http_client.patch(url).json(payload).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn delete_team_folder(&self, team_path: &str, folder_id: &str) -> Result<()> {
        self.retry_request(|| async {
            let url = self.team_folder_url(team_path, folder_id)?;
            let response = self.http_client.delete(url).send().await?;
            self.handle_empty_response(response).await
        })
        .await
    }

    pub async fn get_team_folder_order(&self, team_path: &str) -> Result<FolderOrder> {
        self.retry_request(|| async {
            let url = self.team_folder_order_url(team_path)?;
            let response = self.http_client.get(url).send().await?;
            self.handle_response(response).await
        })
        .await
    }

    pub async fn update_team_folder_order(
        &self,
        team_path: &str,
        payload: &UpdateFolderOrderOptions,
    ) -> Result<()> {
        self.retry_request(|| async {
            let url = self.team_folder_order_url(team_path)?;
            let response = self.http_client.put(url).json(payload).send().await?;
            self.handle_empty_response(response).await
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
        let client = ApiClient::new("   ");
        assert!(client.is_err());

        if let Err(ApiError::MissingRequiredArgument(err)) = client {
            assert!(err.message.contains("Missing access token"));
        } else {
            panic!("Expected MissingRequiredArgument error");
        }
    }

    #[test]
    fn test_api_client_with_base_url() {
        let client = ApiClient::with_base_url("test_token", "https://api.example.com/v1")
            .expect("client should be created");

        assert_eq!(client.base_url.as_str(), "https://api.example.com/v1/");
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
            ..Default::default()
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("Test Note"));
        assert!(json.contains("Test Content"));
    }

    #[test]
    fn test_update_note_options_serialization() {
        let options = UpdateNoteOptions {
            content: Some("Updated content".to_string()),
            write_permission: Some(NotePermissionRole::Guest),
            permalink: Some("custom-permalink".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("Updated content"));
        assert!(json.contains("guest"));
        assert!(json.contains("custom-permalink"));
        // Should not contain null values for None fields
        assert!(!json.contains("readPermission"));
    }

    #[test]
    fn test_note_url_requires_note_id() {
        let client = ApiClient::new("test_token").unwrap();
        let error = client.note_url("   ").unwrap_err();

        assert!(matches!(error, ApiError::MissingRequiredArgument(_)));
    }

    #[test]
    fn test_folder_url_requires_folder_id() {
        let client = ApiClient::new("test_token").unwrap();
        let error = client.folder_url("   ").unwrap_err();

        assert!(matches!(error, ApiError::MissingRequiredArgument(_)));
    }

    #[test]
    fn test_team_note_url_requires_team_path() {
        let client = ApiClient::new("test_token").unwrap();
        let error = client.team_note_url("", "note-123").unwrap_err();

        assert!(matches!(error, ApiError::MissingRequiredArgument(_)));
    }

    #[test]
    fn test_team_folder_url_requires_team_path() {
        let client = ApiClient::new("test_token").unwrap();
        let error = client.team_folder_url("", "folder-123").unwrap_err();

        assert!(matches!(error, ApiError::MissingRequiredArgument(_)));
    }

    #[test]
    fn test_note_team_and_folder_urls_are_composed_from_valid_identifiers() {
        let client = ApiClient::new("test_token").unwrap();

        assert_eq!(
            client.note_url("note-123").unwrap().as_str(),
            "https://api.hackmd.io/v1/notes/note-123"
        );
        assert_eq!(
            client.note_image_url("note-123").unwrap().as_str(),
            "https://api.hackmd.io/v1/notes/note-123/images"
        );
        assert_eq!(
            client.folders_url().unwrap().as_str(),
            "https://api.hackmd.io/v1/folders"
        );
        assert_eq!(
            client.folder_order_url().unwrap().as_str(),
            "https://api.hackmd.io/v1/folders/folder-order"
        );
        assert_eq!(
            client.folder_url("folder-123").unwrap().as_str(),
            "https://api.hackmd.io/v1/folders/folder-123"
        );
        assert_eq!(
            client
                .team_note_url("platform-team", "note-123")
                .unwrap()
                .as_str(),
            "https://api.hackmd.io/v1/teams/platform-team/notes/note-123"
        );
        assert_eq!(
            client.team_folders_url("platform-team").unwrap().as_str(),
            "https://api.hackmd.io/v1/teams/platform-team/folders"
        );
        assert_eq!(
            client
                .team_folder_order_url("platform-team")
                .unwrap()
                .as_str(),
            "https://api.hackmd.io/v1/teams/platform-team/folders/folder-order"
        );
        assert_eq!(
            client
                .team_folder_url("platform-team", "folder-123")
                .unwrap()
                .as_str(),
            "https://api.hackmd.io/v1/teams/platform-team/folders/folder-123"
        );
    }

    #[test]
    fn test_create_folder_options_serialization() {
        let options = CreateFolderOptions {
            name: Some("Project Docs".to_string()),
            parent_folder_id: Some("root-folder".to_string()),
            description: Some("Shared project docs".to_string()),
            icon: Some("📁".to_string()),
            color: Some("#4F46E5".to_string()),
        };

        let json = serde_json::to_string(&options).unwrap();
        assert!(json.contains("Project Docs"));
        assert!(json.contains("root-folder"));
        assert!(json.contains("Shared project docs"));
        assert!(json.contains("📁"));
        assert!(json.contains("#4F46E5"));
    }

    #[test]
    fn test_update_folder_options_serialization_supports_null_clears() {
        let options = UpdateFolderOptions {
            name: Some("Renamed Folder".to_string()),
            parent_folder_id: Some(None),
            description: Some(None),
            icon: None,
            color: Some(Some("#16A34A".to_string())),
        };

        let json = serde_json::to_value(&options).unwrap();
        assert_eq!(json["name"], "Renamed Folder");
        assert_eq!(json["parentFolderId"], Value::Null);
        assert_eq!(json["description"], Value::Null);
        assert_eq!(json["color"], "#16A34A");
        assert!(json.get("icon").is_none());
    }

    #[test]
    fn test_update_folder_order_options_serialization() {
        let options = UpdateFolderOrderOptions {
            order: std::collections::BTreeMap::from([
                (
                    "root".to_string(),
                    vec!["folder-a".to_string(), "folder-b".to_string()],
                ),
                (
                    "folder-a".to_string(),
                    vec!["folder-c".to_string(), "folder-d".to_string()],
                ),
            ]),
        };

        let json = serde_json::to_value(&options).unwrap();
        assert_eq!(
            json["order"]["root"],
            serde_json::json!(["folder-a", "folder-b"])
        );
        assert_eq!(
            json["order"]["folder-a"],
            serde_json::json!(["folder-c", "folder-d"])
        );
    }

    #[test]
    fn test_rate_limit_errors_are_retryable() {
        let client = ApiClient::new("test_token").unwrap();
        let error = ApiError::TooManyRequests(TooManyRequestsError {
            message: "Too many requests".to_string(),
            code: 429,
            status_text: "Too Many Requests".to_string(),
            user_limit: 60,
            user_remaining: 0,
            reset_after: Some(1),
        });

        assert!(client.is_retryable_error(&error));
    }

    #[test]
    fn test_success_status_accepts_all_2xx_codes() {
        assert!(ApiClient::is_success_status(StatusCode::OK));
        assert!(ApiClient::is_success_status(StatusCode::ACCEPTED));
        assert!(ApiClient::is_success_status(StatusCode::NO_CONTENT));
        assert!(!ApiClient::is_success_status(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn test_exponential_backoff_doubles_between_attempts() {
        let client = ApiClient::new("test_token").unwrap();
        let base_delay = time::Duration::from_millis(100);

        assert_eq!(client.exponential_backoff(0, base_delay), base_delay);
        assert_eq!(
            client.exponential_backoff(1, base_delay),
            time::Duration::from_millis(200)
        );
        assert_eq!(
            client.exponential_backoff(2, base_delay),
            time::Duration::from_millis(400)
        );
    }
}
