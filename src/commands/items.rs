use anyhow::{Context, Result};
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::io::{self, Write};
use std::path::Path;

use crate::api::{ApiClient, ItemReadRequest};
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
pub async fn list(json: bool) -> Result<()> {
    let client = ApiClient::new()?;
    let items = client.list_items().await?;

    if json {
        output::print_items_json(&items);
    } else {
        output::print_items_table(&items);
    }

    Ok(())
}

/// Read content from items
/// Format: "id1:1-5,id2:all,id3:10-20"
pub async fn read(ids_str: &str, json: bool) -> Result<()> {
    let items = parse_ids_with_ranges(ids_str)?;

    let client = ApiClient::new()?;
    let response = client.batch_read(items).await?;

    if json {
        output::print_item_content_json(&response.items, &response.not_found);
    } else {
        output::print_item_content(&response.items, &response.not_found);
    }

    Ok(())
}

/// Show table of contents for items
pub async fn toc(ids_str: &str, json: bool) -> Result<()> {
    let ids = parse_ids(ids_str);
    if ids.is_empty() {
        return Err(anyhow::anyhow!("No item IDs provided"));
    }

    let client = ApiClient::new()?;
    let response = client.batch_toc(ids).await?;

    if json {
        output::print_toc_json(&response.items, &response.not_found);
    } else {
        output::print_toc(&response.items, &response.not_found);
    }

    Ok(())
}

/// Upload a PDF file
pub async fn add(file_path: &str) -> Result<()> {
    let path = Path::new(file_path);

    // Validate file exists
    if !path.exists() {
        return Err(anyhow::anyhow!("File not found: {}", file_path));
    }

    // Validate it's a PDF
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    if extension.as_deref() != Some("pdf") {
        return Err(anyhow::anyhow!(
            "Only PDF files are supported. Got: {}",
            extension.unwrap_or_else(|| "no extension".to_string())
        ));
    }

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

    let client = ApiClient::new()?;

    // Step 1: Get presigned upload URL
    print!("{}", "Creating upload...".dimmed());
    io::stdout().flush()?;

    let upload_info = client
        .create_upload(&filename, size, "application/pdf")
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
        .upload_file(&upload_info.upload_url, file_data.clone(), "application/pdf")
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
pub async fn remove(ids_str: &str, skip_confirm: bool) -> Result<()> {
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

    let client = ApiClient::new()?;
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
