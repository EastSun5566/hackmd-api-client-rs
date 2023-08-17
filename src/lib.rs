pub mod types;

use crate::types::User;
use anyhow::{anyhow, Result};
use reqwest::{header, Client as HttpClient, Url};

const DEFAULT_BASE_URL: &str = "https://api.hackmd.io/v1";

pub struct ApiClient {
    http_client: HttpClient,
    // access_token: String,
    base_url: Url,
}

impl ApiClient {
    pub fn new(access_token: String, base_url: Option<String>) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", access_token))?,
        );
        let http_client = HttpClient::builder().default_headers(headers).build()?;

        let base_url: &str = &base_url.unwrap_or(DEFAULT_BASE_URL.to_string());

        Ok(Self {
            http_client,
            // access_token: access_token.to_string(),
            base_url: Url::parse(base_url)?,
        })
    }

    pub async fn get_me(&self) -> Result<User> {
        let url = self.base_url.join("/me")?;
        let res = self.http_client.get(url).send().await?;

        if !res.status().is_success() {
            return Err(anyhow!(res.error_for_status().unwrap_err()));
        }

        let user: User = res.json().await?;
        Ok(user)
    }
}
