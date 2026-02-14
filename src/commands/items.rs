use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::path::Path;

use crate::api::{ApiClient, ItemReadRequest, TocEntry};
use crate::output;

/// Parse comma-separated IDs (for commands that don't use page ranges)
fn parse_ids(ids_str: &str) -> Vec<String> {
    ids_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Parse IDs with page ranges in format "id1:1-5,id2:all,id3:10-20"
/// Every ID must have an explicit range (use 'all' for all pages)
fn parse_ids_with_ranges(ids_str: &str) -> Result<Vec<ItemReadRequest>> {
    let parts: Vec<&str> = ids_str.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if parts.is_empty() {
        return Err(anyhow::anyhow!("No item IDs provided"));
    }

    let mut items = Vec::new();
    let mut missing_ranges: Vec<String> = Vec::new();

    for part in parts {
        if let Some(colon_pos) = part.find(':') {
            let id = part[..colon_pos].trim().to_string();
            let range = part[colon_pos + 1..].trim();

            if id.is_empty() {
                return Err(anyhow::anyhow!("Empty ID found in: '{}'", part));
            }

            // Handle 'all' (case insensitive) as no page filter
            let pages = if range.eq_ignore_ascii_case("all") {
                None
            } else {
                Some(range.to_string())
            };

            items.push(ItemReadRequest { id, pages });
        } else {
            // ID without range - collect for error message
            missing_ranges.push(part.to_string());
        }
    }

    // If any IDs are missing ranges, error with actionable guidance
    if !missing_ranges.is_empty() {
        let examples: Vec<String> = missing_ranges.iter()
            .map(|id| format!("{}:all", id))
            .collect();

        return Err(anyhow::anyhow!(
            "Missing page range for: {}\n\n\
            Every ID must specify a page range. Use 'all' for all pages.\n\
            Example: {}\n\n\
            Formats:\n  \
            • id:all        - All pages\n  \
            • id:1-5        - Pages 1 through 5\n  \
            • id:1,3,5      - Specific pages\n  \
            • id:1-3,7,10   - Combined ranges",
            missing_ranges.join(", "),
            examples.join(",")
        ));
    }

    Ok(items)
}

/// List all items
pub async fn list(json: bool, session: Option<String>, no_session: bool) -> Result<()> {
    let client = ApiClient::new(session, no_session)?;
    let response = client.list_items().await?;

    if json {
        output::print_items_json(&response);
    } else {
        output::print_items_table(&response.items, &response.enrichment_queue);
    }

    Ok(())
}

/// Read content from items
/// Format: "id1:1-5,id2:all,id3:10-20"
pub async fn read(ids_str: &str, json: bool, session: Option<String>, no_session: bool) -> Result<()> {
    let items = parse_ids_with_ranges(ids_str)?;

    let client = ApiClient::new(session, no_session)?;
    let response = client.batch_read(items).await?;

    if json {
        output::print_item_content_json(&response.items, &response.not_found);
    } else {
        output::print_item_content(&response.items, &response.not_found);
    }

    Ok(())
}

/// Show table of contents for items
pub async fn toc(ids_str: &str, json: bool, session: Option<String>, no_session: bool) -> Result<()> {
    let ids = parse_ids(ids_str);
    if ids.is_empty() {
        return Err(anyhow::anyhow!("No item IDs provided"));
    }

    let client = ApiClient::new(session, no_session)?;
    let response = client.batch_toc(ids).await?;

    if json {
        output::print_toc_json(&response.items, &response.not_found);
    } else {
        output::print_toc(&response.items, &response.not_found);
    }

    Ok(())
}

/// Upload a file (PDF or Markdown)
pub async fn add(file_path: &str, session: Option<String>, no_session: bool) -> Result<()> {
    let path = Path::new(file_path);

    // Validate file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    // Get content type based on extension
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    let content_type = match extension.as_deref() {
        Some("pdf") => "application/pdf",
        Some("md") | Some("markdown") => "text/markdown",
        _ => {
            return Err(anyhow::anyhow!(
                "Unsupported file type. Only PDF and Markdown files are supported. Got: {}",
                extension.unwrap_or_else(|| "no extension".to_string())
            ));
        }
    };

    // Get file info
    let metadata = std::fs::metadata(path).context("Failed to read file metadata")?;
    let size = metadata.len();
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?
        .to_string();

    println!("{}", format!("Uploading: {}", filename).cyan());
    println!("{}", format!("Size: {} bytes", size).dimmed());

    let client = ApiClient::new(session, no_session)?;

    // Step 1: Get presigned upload URL
    print!("{}", "Creating upload...".dimmed());
    io::stdout().flush()?;

    let upload_info = client
        .create_upload(&filename, size, content_type)
        .await?;

    println!(" {}", "OK".green());

    // Step 2: Upload file to presigned URL
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})",
        )?
        .progress_chars("#>-"),
    );

    // Read the file
    let file_data = std::fs::read(path).context("Failed to read file")?;

    pb.set_position(0);
    pb.set_message("Uploading...");

    // Upload to presigned URL
    client
        .upload_file(&upload_info.upload_url, file_data.clone(), content_type)
        .await?;

    pb.set_position(size);
    pb.finish_with_message("Upload complete");

    // Step 3: Confirm upload
    print!("{}", "Processing...".dimmed());
    io::stdout().flush()?;

    let confirm = client
        .confirm_upload(&upload_info.item_id, &upload_info.storage_key)
        .await?;

    println!(" {}", "OK".green());

    output::print_success(&format!(
        "Added: {} (ID: {})",
        confirm.item.title,
        confirm.item.id.cyan()
    ));
    output::print_info(&format!(
        "Processing job created: {} ({})",
        confirm.job.id,
        confirm.job.status
    ));

    Ok(())
}

/// Remove items
pub async fn remove(ids_str: &str, skip_confirm: bool, session: Option<String>, no_session: bool) -> Result<()> {
    let ids = parse_ids(ids_str);
    if ids.is_empty() {
        return Err(anyhow::anyhow!("No item IDs provided"));
    }

    // Confirm deletion
    if !skip_confirm {
        println!(
            "{}",
            format!("This will delete {} item(s):", ids.len()).yellow()
        );
        for id in &ids {
            println!("  - {}", id);
        }
        print!("\nAre you sure? [y/N]: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    let client = ApiClient::new(session, no_session)?;
    let response = client.delete_items(ids).await?;

    // Report results
    if !response.deleted.is_empty() {
        output::print_success(&format!(
            "Deleted {} item(s): {}",
            response.deleted.len(),
            response.deleted.join(", ")
        ));
    }

    if !response.not_found.is_empty() {
        output::print_warning(&format!(
            "Not found: {}",
            response.not_found.join(", ")
        ));
    }

    if let Some(storage_errors) = response.storage_errors {
        if !storage_errors.is_empty() {
            output::print_warning(&format!(
                "Storage cleanup failed for: {}",
                storage_errors.join(", ")
            ));
        }
    }

    Ok(())
}

/// Enrich item metadata
pub async fn enrich(
    id: &str,
    title: Option<&str>,
    author: Option<&str>,
    description: Option<&str>,
    confidence: Option<f64>,
    toc_json: Option<&str>,
    session: Option<String>,
    no_session: bool,
) -> Result<()> {
    if title.is_none() && author.is_none() && description.is_none() && toc_json.is_none() {
        return Err(anyhow::anyhow!(
            "At least one of --title, --author, --description, or --toc is required"
        ));
    }

    if let Some(conf) = confidence {
        if !(0.0..=1.0).contains(&conf) {
            return Err(anyhow::anyhow!("Confidence must be between 0.0 and 1.0"));
        }
    }

    // Parse TOC JSON if provided
    let toc: Option<Vec<TocEntry>> = match toc_json {
        Some(json_str) => {
            let parsed: Vec<TocEntry> = serde_json::from_str(json_str)
                .context("Invalid TOC JSON. Expected format: [{\"title\":\"Chapter 1\",\"page\":1,\"level\":1}]")?;

            // Validate TOC entries
            for entry in &parsed {
                if entry.title.trim().is_empty() {
                    return Err(anyhow::anyhow!("TOC entry title cannot be empty"));
                }
                if entry.page < 1 {
                    return Err(anyhow::anyhow!("TOC entry page must be >= 1"));
                }
                if let Some(level) = entry.level {
                    if level < 1 {
                        return Err(anyhow::anyhow!("TOC entry level must be >= 1"));
                    }
                }
            }

            Some(parsed)
        }
        None => None,
    };

    let client = ApiClient::new(session, no_session)?;
    let response = client.enrich_item(id, title, author, description, confidence, toc.clone()).await?;

    output::print_success(&format!(
        "Enriched: {} (ID: {})",
        response.item.title,
        response.item.id.cyan()
    ));

    if let Some(author) = &response.item.author {
        output::print_info(&format!("Author: {}", author));
    }

    if let Some(desc) = &response.item.description {
        let preview = if desc.len() > 80 {
            format!("{}...", &desc[..77])
        } else {
            desc.clone()
        };
        output::print_info(&format!("Description: {}", preview));
    }

    if let Some(ref toc_entries) = toc {
        output::print_info(&format!("TOC: {} entries added", toc_entries.len()));
    }

    if let Some(conf) = response.item.enrichment_confidence {
        output::print_info(&format!("Confidence: {:.1}%", conf * 100.0));
    }

    if response.item.needs_enrichment {
        output::print_warning("Still flagged for enrichment (confidence < 80%)");
    }

    Ok(())
}

/// Flag item as needing enrichment
pub async fn flag(id: &str, session: Option<String>, no_session: bool) -> Result<()> {
    let client = ApiClient::new(session, no_session)?;
    let response = client.flag_item(id).await?;

    output::print_success(&format!(
        "Flagged for enrichment: {} (ID: {})",
        response.item.title,
        response.item.id.cyan()
    ));

    Ok(())
}

/// Create a new markdown document
pub async fn create(
    title: &str,
    description: Option<&str>,
    content: Option<&str>,
    json: bool,
    session: Option<String>,
    no_session: bool,
) -> Result<()> {
    let client = ApiClient::new(session, no_session)?;
    let response = client.create_markdown(title, description, content).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        output::print_success(&format!(
            "Created: {} (ID: {})",
            response.title,
            response.id.cyan()
        ));
        println!("  Pages: {}", response.page_count);
        println!();
        println!("  To add content: ck items put {} --file content.md", response.id);
        println!("  To view:        ck items get {}", response.id);
    }

    Ok(())
}

/// Get full content of a document (outputs to stdout for piping)
pub async fn get(id: &str, session: Option<String>, no_session: bool) -> Result<()> {
    let client = ApiClient::new(session, no_session)?;
    let response = client.get_content(id).await?;

    // Output raw content to stdout (for piping to files)
    print!("{}", response.content);

    Ok(())
}

/// Replace document content from file or stdin
pub async fn put(id: &str, file_path: Option<&str>, session: Option<String>, no_session: bool) -> Result<()> {
    let content = if let Some(path) = file_path {
        // Read from file
        let path = Path::new(path);
        if !path.exists() {
            return Err(anyhow::anyhow!("File not found: {}", path.display()));
        }
        std::fs::read_to_string(path).context("Failed to read file")?
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin()
            .read_line(&mut buffer)
            .context("Failed to read from stdin")?;

        // Keep reading until EOF
        loop {
            let mut line = String::new();
            match io::stdin().read_line(&mut line) {
                Ok(0) => break, // EOF
                Ok(_) => buffer.push_str(&line),
                Err(e) => return Err(anyhow::anyhow!("Failed to read from stdin: {}", e)),
            }
        }
        buffer
    };

    if content.trim().is_empty() {
        return Err(anyhow::anyhow!("No content provided"));
    }

    let client = ApiClient::new(session, no_session)?;
    let response = client.put_content(id, &content).await?;

    output::print_success(&format!(
        "Updated: {} (ID: {})",
        response.title,
        response.id.cyan()
    ));
    println!("  Version: {}", response.version);
    println!("  Pages: {}", response.page_count);

    Ok(())
}
