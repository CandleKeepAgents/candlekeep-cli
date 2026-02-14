use anyhow::Result;

use crate::api::ApiClient;

pub async fn start(
    intent: Option<&str>,
    json: bool,
    session: Option<String>,
    no_session: bool,
) -> Result<()> {
    let client = match ApiClient::new(session, no_session) {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{{\"error\":\"{}\"}}", e);
            } else {
                eprintln!("Warning: {}", e);
            }
            return Ok(());
        }
    };

    match client.create_session(intent).await {
        Ok(resp) => {
            // Write session ID to file
            if let Err(e) = ApiClient::write_session_file(&resp.session_id) {
                eprintln!("Warning: Failed to write session file: {}", e);
            }

            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Session started: {}", resp.session_id);
            }
        }
        Err(e) => {
            // Tracking failure must NEVER block research
            if json {
                println!("{{\"error\":\"{}\"}}", e);
            } else {
                eprintln!("Warning: Failed to start session (continuing without tracking)");
                eprintln!("  {}", e);
            }
        }
    }

    Ok(())
}

pub async fn complete(
    json: bool,
    session: Option<String>,
    no_session: bool,
) -> Result<()> {
    // Resolve session ID: --session flag > file
    let session_id = if let Some(ref s) = session {
        s.clone()
    } else {
        match ApiClient::read_session_file() {
            Some(s) => s,
            None => {
                if json {
                    println!("{{\"error\":\"No active session\"}}");
                } else {
                    eprintln!("No active session found");
                }
                return Ok(());
            }
        }
    };

    let client = match ApiClient::new(session, no_session) {
        Ok(c) => c,
        Err(e) => {
            if json {
                println!("{{\"error\":\"{}\"}}", e);
            } else {
                eprintln!("Warning: {}", e);
            }
            return Ok(());
        }
    };

    match client.complete_session(&session_id).await {
        Ok(resp) => {
            // Delete session file
            ApiClient::delete_session_file();

            if json {
                println!("{}", serde_json::to_string_pretty(&resp)?);
            } else {
                println!("Session completed: {}", resp.session_id);
            }
        }
        Err(e) => {
            // Still delete the file on failure
            ApiClient::delete_session_file();

            if json {
                println!("{{\"error\":\"{}\"}}", e);
            } else {
                eprintln!("Warning: Failed to complete session: {}", e);
            }
        }
    }

    Ok(())
}
