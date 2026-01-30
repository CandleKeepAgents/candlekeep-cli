use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpListener;

use crate::api::ApiClient;
use crate::config;
use crate::output;

/// Login via browser authentication
pub async fn login() -> Result<()> {
    // Check if already authenticated
    if config::is_authenticated() {
        output::print_warning("Already logged in. Use 'ck auth logout' first to re-authenticate.");
        return Ok(());
    }

    // Bind to a random available port
    let listener = TcpListener::bind("127.0.0.1:0").context("Failed to start local server")?;
    let port = listener.local_addr()?.port();

    let api_url = config::get_api_url()?;
    let auth_url = format!("{}/cli-auth?port={}", api_url, port);

    println!("{}", "Opening browser for authentication...".cyan());
    println!("If browser doesn't open, visit: {}", auth_url.underline());

    // Try to open the browser
    if open::that(&auth_url).is_err() {
        println!(
            "\n{}",
            "Could not open browser automatically.".yellow()
        );
    }

    println!("\n{}", "Waiting for authorization...".dimmed());

    // Accept the callback
    let api_key = match wait_for_callback(&listener).await {
        Ok(key) => key,
        Err(e) => {
            // Fallback to manual key entry
            println!("\n{}", "Browser authentication failed.".yellow());
            println!("{}", e);
            return manual_key_entry().await;
        }
    };

    // Validate the key
    validate_and_save_key(&api_key).await
}

async fn wait_for_callback(listener: &TcpListener) -> Result<String> {
    // Set timeout for accepting connections
    listener.set_nonblocking(false)?;

    // Use a thread to handle the TCP listener since it's blocking
    let listener_clone = listener.try_clone()?;
    let handle = std::thread::spawn(move || -> Result<String> {
        // Accept connection with timeout (60 seconds)
        let (mut stream, _) = listener_clone.accept()?;

        // Read the request
        let mut reader = BufReader::new(&stream);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;

        // Parse the key from the request
        // Expected: GET /callback?key=ck_xxx HTTP/1.1
        let api_key = request_line
            .split_whitespace()
            .nth(1)
            .and_then(|path| path.strip_prefix("/callback?key="))
            .map(|s| s.to_string())
            .context("Invalid callback URL")?;

        // Send success response
        let response = r#"HTTP/1.1 200 OK
Content-Type: text/html; charset=utf-8
Connection: close

<!DOCTYPE html>
<html>
<head>
    <title>CandleKeep CLI</title>
    <style>
        body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; display: flex; justify-content: center; align-items: center; height: 100vh; margin: 0; background: #1a1a1a; color: #fff; }
        .container { text-align: center; padding: 2rem; }
        .success { color: #22c55e; font-size: 3rem; margin-bottom: 1rem; }
        h1 { margin: 0 0 0.5rem 0; }
        p { color: #888; }
    </style>
</head>
<body>
    <div class="container">
        <div class="success">✓</div>
        <h1>Authentication Successful</h1>
        <p>You can close this window and return to the terminal.</p>
    </div>
</body>
</html>"#;

        use std::io::Write;
        stream.write_all(response.as_bytes())?;
        stream.flush()?;

        Ok(api_key)
    });

    // Wait for the thread with a timeout
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(anyhow::anyhow!("Callback handler panicked")),
    }
}

async fn manual_key_entry() -> Result<()> {
    println!("\nTo authenticate manually:");
    println!("1. Go to {} and log in", config::get_api_url()?.underline());
    println!("2. Navigate to Settings > API Keys");
    println!("3. Create a new API key and copy it");
    println!();

    print!("Enter your API key: ");
    io::stdout().flush()?;

    let mut api_key = String::new();
    io::stdin().read_line(&mut api_key)?;
    let api_key = api_key.trim().to_string();

    if api_key.is_empty() {
        return Err(anyhow::anyhow!("No API key provided"));
    }

    validate_and_save_key(&api_key).await
}

async fn validate_and_save_key(api_key: &str) -> Result<()> {
    print!("{}", "Validating API key...".dimmed());
    io::stdout().flush()?;

    // Validate the key by calling whoami
    let client = ApiClient::with_key(api_key)?;
    let user = client.whoami().await.context("Invalid API key")?;

    println!(" {}", "OK".green());

    // Save the key
    config::save_api_key(api_key)?;

    output::print_success(&format!(
        "Logged in as {} ({})",
        user.email.cyan(),
        user.tier
    ));

    Ok(())
}

/// Logout - remove stored credentials
pub fn logout() -> Result<()> {
    if !config::is_authenticated() {
        output::print_warning("Not currently logged in.");
        return Ok(());
    }

    config::clear_config()?;
    output::print_success("Logged out successfully.");
    Ok(())
}

/// Show current user information
pub async fn whoami(json: bool) -> Result<()> {
    let client = ApiClient::new()?;
    let user = client.whoami().await?;

    if json {
        output::print_whoami_json(&user);
    } else {
        output::print_whoami(&user);
    }

    Ok(())
}
