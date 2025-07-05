//! HTTP client library for making API requests.
//!
//! This library provides a simplified wrapper around reqwest for making
//! HTTP requests with JSON serialization/deserialization support.
//!
//! # Examples
//!
//! ```rust
//! use ej_requests::ApiClient;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let client = ApiClient::new("https://api.example.com");
//! let data: serde_json::Value = client.get("users").await;
//! # Ok(())
//! # }
//! ```

use std::{borrow::Borrow, error::Error, str::FromStr};

use reqwest::{Response, StatusCode, Url, header};
use serde::de::DeserializeOwned;

/// HTTP client for making API requests with JSON support.
pub struct ApiClient {
    url: String,
    pub client: reqwest::Client,
}

impl ApiClient {
    /// Creates a new API client with the given base URL.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use ej_requests::ApiClient;
    ///
    /// let client = ApiClient::new("https://api.example.com");
    /// ```
    pub fn new(url: impl Into<String>) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "content-type",
            header::HeaderValue::from_static("application/json"),
        );
        let client = reqwest::ClientBuilder::new()
            .default_headers(headers)
            .cookie_store(true)
            .build()
            .expect("Failed to build reqwest Client");
        Self {
            url: url.into(),
            client,
        }
    }

    /// Constructs the full URL path for an endpoint.
    fn path(&self, endpoint: &str) -> String {
        format!("{}/{endpoint}", self.url)
    }

    /// Makes a GET request to the specified URL and deserializes the response.
    async fn get_url<T: DeserializeOwned>(url: Url) -> T {
        serde_json::from_str(
            &reqwest::get(url)
                .await
                .expect("Failed to send http request")
                .text()
                .await
                .expect("Failed to get response text"),
        )
        .expect("Couldn't Parse Value")
    }

    /// Makes a GET request to the specified endpoint.
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> T {
        let url = reqwest::Url::from_str(&self.path(&endpoint)).unwrap();
        Self::get_url(url).await
    }

    /// Makes a GET request with query parameters.
    pub async fn get_with_body<T, I, K, V>(&self, endpoint: &str, params: I) -> T
    where
        T: DeserializeOwned,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let url = reqwest::Url::parse_with_params(&self.path(&endpoint), params)
            .expect("Couldn't create get request");
        Self::get_url(url).await
    }

    /// Makes a POST request with the given body.
    pub async fn post<T: Into<reqwest::Body>>(
        &self,
        endpoint: &str,
        body: T,
    ) -> Result<Response, Box<dyn Error>> {
        let url = reqwest::Url::from_str(&self.path(endpoint)).unwrap();
        Ok(self
            .client
            .post(url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?)
    }

    /// Makes a POST request and deserializes the response.
    pub async fn post_and_deserialize<T: Into<reqwest::Body>, U: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: T,
    ) -> Result<U, Box<dyn Error>> {
        let url = reqwest::Url::from_str(&self.path(endpoint)).unwrap();

        let response = self
            .client
            .post(url)
            .header("content-type", "application/json")
            .body(body)
            .send()
            .await?
            .text()
            .await?;

        Ok(serde_json::from_str(&response)?)
    }

    /// Makes a POST request without a body and deserializes the response.
    pub async fn post_no_body<T: DeserializeOwned>(
        &self,
        endpoint: &str,
    ) -> Result<T, Box<dyn Error>> {
        let url = reqwest::Url::from_str(&self.path(endpoint)).unwrap();

        let response = self.client.post(url).send().await?.text().await?;

        println!("Response {response}");
        Ok(serde_json::from_str(&response)?)
    }

    /// Makes a DELETE request with query parameters.
    pub async fn delete<I, K, V>(
        &self,
        client: &reqwest::Client,
        endpoint: &str,
        params: I,
    ) -> StatusCode
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let url = reqwest::Url::parse_with_params(&self.path(&endpoint), params)
            .expect("Couldn't create get request");
        client
            .delete(url)
            .header("content-type", "application/json")
            .send()
            .await
            .expect("Failed to send patch request")
            .status()
    }
}
