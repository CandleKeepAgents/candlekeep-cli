#![allow(dead_code)]

use anyhow::{Context, Result};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};

use crate::config;

/// API client for CandleKeep
pub struct ApiClient {
    client: Client,
    base_url: String,
    api_key: String,
}

// Response types

#[derive(Debug, Deserialize, Serialize)]
pub struct WhoamiResponse {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
    pub tier: String,
    #[serde(rename = "itemLimit")]
    pub item_limit: i32,
    #[serde(rename = "itemCount")]
    pub item_count: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EnrichmentQueueItem {
    pub id: String,
    pub title: String,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemsResponse {
    pub items: Vec<Item>,
    #[serde(rename = "enrichmentQueue")]
    pub enrichment_queue: Option<Vec<EnrichmentQueueItem>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Item {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub author: Option<String>,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    #[serde(rename = "needsEnrichment")]
    pub needs_enrichment: Option<bool>,
    #[serde(rename = "enrichmentConfidence")]
    pub enrichment_confidence: Option<f64>,
    #[serde(rename = "enrichedAt")]
    pub enriched_at: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
    #[serde(rename = "latestJob")]
    pub latest_job: Option<Job>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Job {
    pub id: String,
    #[serde(rename = "type")]
    pub job_type: String,
    pub status: String,
    pub progress: Option<i32>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchItemsResponse {
    pub items: Vec<ItemWithPages>,
    #[serde(rename = "notFound")]
    pub not_found: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemWithPages {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    pub metadata: Option<serde_json::Value>,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
    pub pages: Vec<Page>,
    #[serde(rename = "latestJob")]
    pub latest_job: Option<Job>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Page {
    pub id: String,
    #[serde(rename = "pageNum")]
    pub page_num: i32,
    pub content: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct BatchTocResponse {
    pub items: Vec<ItemWithToc>,
    #[serde(rename = "notFound")]
    pub not_found: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ItemWithToc {
    pub id: String,
    pub title: String,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
    pub toc: Option<Vec<TocEntry>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TocEntry {
    pub title: String,
    pub page: i32,
    pub level: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct UploadResponse {
    #[serde(rename = "itemId")]
    pub item_id: String,
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,
    #[serde(rename = "storageKey")]
    pub storage_key: String,
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmResponse {
    pub item: ConfirmItem,
    pub job: ConfirmJob,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmItem {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmJob {
    pub id: String,
    #[serde(rename = "type")]
    pub job_type: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteResponse {
    pub deleted: Vec<String>,
    #[serde(rename = "notFound")]
    pub not_found: Vec<String>,
    #[serde(rename = "storageErrors")]
    pub storage_errors: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Debug, Deserialize)]
pub struct EnrichResponse {
    pub item: EnrichedItem,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EnrichedItem {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "needsEnrichment")]
    pub needs_enrichment: bool,
    #[serde(rename = "enrichmentConfidence")]
    pub enrichment_confidence: Option<f64>,
    #[serde(rename = "enrichedAt")]
    pub enriched_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FlagResponse {
    pub item: FlaggedItem,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FlaggedItem {
    pub id: String,
    pub title: String,
    #[serde(rename = "needsEnrichment")]
    pub needs_enrichment: bool,
}

// Markdown document types

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateMarkdownResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    #[serde(rename = "sourceType")]
    pub source_type: String,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GetContentResponse {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub content: String,
    pub version: i32,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PutContentResponse {
    pub id: String,
    pub title: String,
    pub version: i32,
    #[serde(rename = "pageCount")]
    pub page_count: i32,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// Request type for reading items with optional per-item page ranges
#[derive(Debug, Serialize)]
pub struct ItemReadRequest {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pages: Option<String>,
}

impl ApiClient {
    /// Create a new API client with the configured API key
    pub fn new() -> Result<Self> {
        let api_key = config::get_api_key()?
            .context("Not authenticated. Run 'ck auth login' first.")?;
        let base_url = config::get_api_url()?;

        let client = Client::builder()
            .user_agent(format!("ck-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            api_key,
        })
    }

    /// Create a new API client with a specific API key (for validation)
    pub fn with_key(api_key: &str) -> Result<Self> {
        let base_url = config::get_api_url()?;

        let client = Client::builder()
            .user_agent(format!("ck-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            base_url,
            api_key: api_key.to_string(),
        })
    }

    fn api_url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    /// Handle API error responses
    async fn handle_error(response: reqwest::Response) -> anyhow::Error {
        let status = response.status();
        let error_text = response
            .json::<ApiError>()
            .await
            .map(|e| e.error)
            .unwrap_or_else(|_| format!("HTTP {}", status));

        match status {
            StatusCode::UNAUTHORIZED => anyhow::anyhow!("Authentication failed: {}", error_text),
            StatusCode::FORBIDDEN => anyhow::anyhow!("Access denied: {}", error_text),
            StatusCode::NOT_FOUND => anyhow::anyhow!("Not found: {}", error_text),
            StatusCode::BAD_REQUEST => anyhow::anyhow!("Bad request: {}", error_text),
            _ => anyhow::anyhow!("API error ({}): {}", status, error_text),
        }
    }

    /// GET /api/v1/auth/whoami
    pub async fn whoami(&self) -> Result<WhoamiResponse> {
        let response = self
            .client
            .get(self.api_url("/auth/whoami"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// GET /api/v1/items
    pub async fn list_items(&self) -> Result<ItemsResponse> {
        let response = self
            .client
            .get(self.api_url("/items"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// POST /api/v1/items/batch - Get multiple items with their pages
    /// Supports per-item page ranges via the new `items` format
    pub async fn batch_read(&self, items: Vec<ItemReadRequest>) -> Result<BatchItemsResponse> {
        #[derive(Serialize)]
        struct Body {
            items: Vec<ItemReadRequest>,
        }

        let response = self
            .client
            .post(self.api_url("/items/batch"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body { items })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// POST /api/v1/items/batch/toc - Get table of contents for multiple items
    pub async fn batch_toc(&self, ids: Vec<String>) -> Result<BatchTocResponse> {
        #[derive(Serialize)]
        struct Body {
            ids: Vec<String>,
        }

        let response = self
            .client
            .post(self.api_url("/items/batch/toc"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body { ids })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// POST /api/v1/upload - Get presigned URL for file upload
    pub async fn create_upload(
        &self,
        filename: &str,
        size: u64,
        content_type: &str,
    ) -> Result<UploadResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            filename: &'a str,
            size: u64,
            #[serde(rename = "contentType")]
            content_type: &'a str,
        }

        let response = self
            .client
            .post(self.api_url("/upload"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body {
                filename,
                size,
                content_type,
            })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// Upload file to presigned URL
    pub async fn upload_file(&self, url: &str, data: Vec<u8>, content_type: &str) -> Result<()> {
        let response = self
            .client
            .put(url)
            .header("Content-Type", content_type)
            .body(data)
            .send()
            .await
            .context("Failed to upload file")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Upload failed ({}): {}", status, text));
        }

        Ok(())
    }

    /// POST /api/v1/upload/confirm - Confirm upload and create processing job
    pub async fn confirm_upload(&self, item_id: &str, storage_key: &str) -> Result<ConfirmResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "itemId")]
            item_id: &'a str,
            #[serde(rename = "storageKey")]
            storage_key: &'a str,
        }

        let response = self
            .client
            .post(self.api_url("/upload/confirm"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body { item_id, storage_key })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// DELETE /api/v1/items - Delete multiple items
    pub async fn delete_items(&self, ids: Vec<String>) -> Result<DeleteResponse> {
        #[derive(Serialize)]
        struct Body {
            ids: Vec<String>,
        }

        let response = self
            .client
            .delete(self.api_url("/items"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body { ids })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// PATCH /api/v1/items/enrich - Enrich item metadata
    pub async fn enrich_item(
        &self,
        item_id: &str,
        title: Option<&str>,
        author: Option<&str>,
        description: Option<&str>,
        confidence: Option<f64>,
        toc: Option<Vec<TocEntry>>,
    ) -> Result<EnrichResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "itemId")]
            item_id: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            author: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            confidence: Option<f64>,
            #[serde(skip_serializing_if = "Option::is_none")]
            toc: Option<Vec<TocEntry>>,
        }

        let response = self
            .client
            .patch(self.api_url("/items/enrich"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body {
                item_id,
                title,
                author,
                description,
                confidence,
                toc,
            })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// POST /api/v1/items/flag - Flag item as needing enrichment
    pub async fn flag_item(&self, item_id: &str) -> Result<FlagResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            #[serde(rename = "itemId")]
            item_id: &'a str,
        }

        let response = self
            .client
            .post(self.api_url("/items/flag"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body { item_id })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// POST /api/v1/items/markdown - Create a new markdown document
    pub async fn create_markdown(
        &self,
        title: &str,
        description: Option<&str>,
        content: Option<&str>,
    ) -> Result<CreateMarkdownResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            title: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            content: Option<&'a str>,
        }

        let response = self
            .client
            .post(self.api_url("/items/markdown"))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body {
                title,
                description,
                content,
            })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// GET /api/v1/items/:id/content - Get full document content
    pub async fn get_content(&self, item_id: &str) -> Result<GetContentResponse> {
        let response = self
            .client
            .get(self.api_url(&format!("/items/{}/content", item_id)))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }

    /// PUT /api/v1/items/:id/content - Replace document content
    pub async fn put_content(&self, item_id: &str, content: &str) -> Result<PutContentResponse> {
        #[derive(Serialize)]
        struct Body<'a> {
            content: &'a str,
        }

        let response = self
            .client
            .put(self.api_url(&format!("/items/{}/content", item_id)))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&Body { content })
            .send()
            .await
            .context("Failed to connect to API")?;

        if !response.status().is_success() {
            return Err(Self::handle_error(response).await);
        }

        response
            .json()
            .await
            .context("Failed to parse response")
    }
}
