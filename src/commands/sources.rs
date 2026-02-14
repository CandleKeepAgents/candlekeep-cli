use anyhow::Result;
use colored::Colorize;
use std::io::{self, Write};

use crate::api::ApiClient;
use crate::output;

/// Parse comma-separated IDs
fn parse_ids(ids_str: &str) -> Vec<String> {
    ids_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// List sources
pub async fn list(json: bool, limit: Option<u32>, session: Option<String>, no_session: bool) -> Result<()> {
    let client = ApiClient::new(session, no_session)?;
    let limit = limit.unwrap_or(50);
    let response = client.list_sources(limit, None).await?;

    if json {
        output::print_sources_json(&response);
    } else {
        output::print_sources_table(&response.sources, response.total);
    }

    Ok(())
}

/// Delete sources
pub async fn delete(ids_str: &str, skip_confirm: bool, session: Option<String>, no_session: bool) -> Result<()> {
    let ids = parse_ids(ids_str);
    if ids.is_empty() {
        return Err(anyhow::anyhow!("No source IDs provided"));
    }

    // Confirm deletion
    if !skip_confirm {
        println!(
            "{}",
            format!("This will delete {} source(s):", ids.len()).yellow()
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
    let response = client.delete_sources(ids).await?;

    // Report results
    if !response.deleted.is_empty() {
        output::print_success(&format!(
            "Deleted {} source(s): {}",
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

    Ok(())
}
