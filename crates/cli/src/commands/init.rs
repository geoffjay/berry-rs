//! Init command - initialize configuration.

use anyhow::Result;

use berry::config::{config_path, ensure_config_dir};

use crate::output::{print_info, print_success};

/// Init command arguments.
#[derive(Debug)]
pub struct InitArgs {
    pub force: bool,
}

/// Run the init command.
pub async fn run(args: InitArgs) -> Result<()> {
    // Ensure config directory exists
    let config_dir = ensure_config_dir()?;
    print_info(&format!("Config directory: {}", config_dir.display()));

    // Check if config file already exists
    if let Some(path) = config_path()
        && path.exists()
        && !args.force
    {
        print_info(&format!(
            "Config file already exists at: {}",
            path.display()
        ));
        print_info("Use --force to overwrite.");
        return Ok(());
    }

    // Write default config
    let path = write_default_config()?;
    print_success(&format!("Created config file: {}", path.display()));

    Ok(())
}

/// Write a default configuration file.
fn write_default_config() -> Result<std::path::PathBuf> {
    let dir = ensure_config_dir()?;
    let path = dir.join("config.jsonc");

    let content = r#"{
  // Berry Configuration
  // See https://github.com/geoffjay/berry-rs for documentation

  "server": {
    // URL of the Berry server
    "url": "http://localhost:4114",
    // Request timeout in milliseconds
    "timeout": 5000
  },

  "defaults": {
    // Default memory type: question, request, information
    "type": "information",
    // Default creator identifier
    "createdBy": "user",
    // Default visibility: private, shared, public
    "visibility": "public"
  },

  "chroma": {
    // ChromaDB server URL
    "url": "http://localhost:8000",
    // Collection name for storing memories
    "collection": "berry_memories"
    // Optional: authentication provider
    // "provider": "token",
    // Optional: API key for authentication
    // "apiKey": "your-api-key"
  }
}
"#;

    std::fs::write(&path, content)?;
    Ok(path)
}
