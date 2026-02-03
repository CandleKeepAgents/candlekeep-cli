#![allow(dead_code)]

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

use crate::api::{EnrichmentQueueItem, Item, ItemsResponse, ItemWithPages, ItemWithToc, TocEntry, WhoamiResponse};

/// Status color mapping
fn status_color(status: &str) -> Color {
    match status.to_uppercase().as_str() {
        "COMPLETED" | "SUCCESS" => Color::Green,
        "PENDING" | "QUEUED" => Color::Yellow,
        "PROCESSING" | "RUNNING" => Color::Cyan,
        "FAILED" | "ERROR" => Color::Red,
        _ => Color::White,
    }
}

/// Format job status with color
pub fn format_status(status: &str) -> String {
    match status.to_uppercase().as_str() {
        "COMPLETED" | "SUCCESS" => status.green().to_string(),
        "PENDING" | "QUEUED" => status.yellow().to_string(),
        "PROCESSING" | "RUNNING" => status.cyan().to_string(),
        "FAILED" | "ERROR" => status.red().to_string(),
        _ => status.to_string(),
    }
}

/// Print user info as table
pub fn print_whoami(info: &WhoamiResponse) {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);

    table.add_row(vec!["Email", &info.email]);
    if let Some(ref name) = info.name {
        table.add_row(vec!["Name", name]);
    }
    table.add_row(vec!["Tier", &info.tier]);
    table.add_row(vec![
        "Items",
        &format!("{} / {}", info.item_count, info.item_limit),
    ]);
    table.add_row(vec!["User ID", &info.id]);

    println!("{table}");
}

/// Print user info as JSON
pub fn print_whoami_json(info: &WhoamiResponse) {
    println!("{}", serde_json::to_string_pretty(info).unwrap());
}

/// Print items as table
pub fn print_items_table(items: &[Item], enrichment_queue: &Option<Vec<EnrichmentQueueItem>>) {
    if items.is_empty() {
        println!("{}", "No items found.".dimmed());
        return;
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("ID").fg(Color::Cyan),
            Cell::new("Title").fg(Color::Cyan),
            Cell::new("Pages").fg(Color::Cyan),
            Cell::new("Status").fg(Color::Cyan),
            Cell::new("Enrich").fg(Color::Cyan),
        ]);

    for item in items {
        let status = item
            .latest_job
            .as_ref()
            .map(|j| j.status.clone())
            .unwrap_or_else(|| "N/A".to_string());

        let status_cell = Cell::new(&status).fg(status_color(&status));

        let enrich_status = if item.needs_enrichment.unwrap_or(false) {
            Cell::new("⚠").fg(Color::Yellow)
        } else if item.enrichment_confidence.is_some() {
            Cell::new("✓").fg(Color::Green)
        } else {
            Cell::new("-").fg(Color::DarkGrey)
        };

        table.add_row(vec![
            Cell::new(&item.id),
            Cell::new(&item.title),
            Cell::new(item.page_count),
            status_cell,
            enrich_status,
        ]);
    }

    println!("{table}");
    println!(
        "\n{} {}",
        items.len().to_string().bold(),
        if items.len() == 1 { "item" } else { "items" }
    );

    // Print enrichment queue if present
    if let Some(queue) = enrichment_queue {
        if !queue.is_empty() {
            println!();
            println!("{}", "Enrichment Queue:".yellow().bold());
            for item in queue {
                println!(
                    "  {} {} ({} pages)",
                    "⚠".yellow(),
                    item.title.dimmed(),
                    item.page_count
                );
            }
        }
    }
}

/// Print items as JSON
pub fn print_items_json(response: &ItemsResponse) {
    println!("{}", serde_json::to_string_pretty(response).unwrap());
}

/// Print item content with page numbers
/// Output format is designed to be clean for both terminal use and agent consumption.
/// The markdown content is printed raw, allowing agents to read it directly.
pub fn print_item_content(items: &[ItemWithPages], not_found: &Option<Vec<String>>) {
    for item in items {
        // Header section with book info
        println!();
        println!("{}", "─".repeat(60).dimmed());
        println!("{}", item.title.bold().cyan());
        println!(
            "{} | {} pages",
            format!("ID: {}", item.id).dimmed(),
            item.page_count
        );
        println!("{}", "─".repeat(60).dimmed());

        if item.pages.is_empty() {
            println!("{}", "No pages available.".yellow());
            continue;
        }

        for page in &item.pages {
            // Page separator - clean format that works in markdown and terminal
            println!();
            println!("{}", format!("── Page {} ──", page.page_num).blue().bold());
            println!();

            // Output raw markdown content (no transformation)
            if let Some(ref content) = page.content {
                println!("{}", content);
            } else {
                println!("{}", "(No content)".dimmed());
            }
        }
    }

    if let Some(ref not_found_ids) = not_found {
        if !not_found_ids.is_empty() {
            println!(
                "\n{}: {}",
                "Items not found".yellow(),
                not_found_ids.join(", ")
            );
        }
    }
}

/// Print item content as JSON
pub fn print_item_content_json(items: &[ItemWithPages], not_found: &Option<Vec<String>>) {
    #[derive(serde::Serialize)]
    struct Output<'a> {
        items: &'a [ItemWithPages],
        #[serde(skip_serializing_if = "Option::is_none")]
        not_found: &'a Option<Vec<String>>,
    }

    let output = Output { items, not_found };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Print table of contents
pub fn print_toc(items: &[ItemWithToc], not_found: &Option<Vec<String>>) {
    for item in items {
        println!(
            "\n{} {}",
            "=".repeat(60).dimmed(),
            item.title.bold().cyan()
        );
        println!(
            "{} | {} pages",
            format!("ID: {}", item.id).dimmed(),
            item.page_count
        );
        println!("{}", "=".repeat(80).dimmed());

        match &item.toc {
            Some(toc) if !toc.is_empty() => {
                print_toc_entries(toc);
            }
            _ => {
                println!("{}", "No table of contents available.".yellow());
            }
        }
    }

    if let Some(ref not_found_ids) = not_found {
        if !not_found_ids.is_empty() {
            println!(
                "\n{}: {}",
                "Items not found".yellow(),
                not_found_ids.join(", ")
            );
        }
    }
}

fn print_toc_entries(entries: &[TocEntry]) {
    for entry in entries {
        let indent = "  ".repeat(entry.level.unwrap_or(0) as usize);
        println!(
            "{}{}{}",
            indent,
            entry.title,
            format!(" (p. {})", entry.page).dimmed()
        );
    }
}

/// Print TOC as JSON
pub fn print_toc_json(items: &[ItemWithToc], not_found: &Option<Vec<String>>) {
    #[derive(serde::Serialize)]
    struct Output<'a> {
        items: &'a [ItemWithToc],
        #[serde(skip_serializing_if = "Option::is_none")]
        not_found: &'a Option<Vec<String>>,
    }

    let output = Output { items, not_found };
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

/// Print success message
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print error message
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print warning message
pub fn print_warning(message: &str) {
    println!("{} {}", "!".yellow().bold(), message);
}

/// Print info message
pub fn print_info(message: &str) {
    println!("{} {}", "i".cyan().bold(), message);
}
