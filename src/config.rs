use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const CONFIG_DIR: &str = ".nsg";
const CREDENTIALS_FILE: &str = "credentials.json";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub username: String,
    pub password: String,
    pub app_key: String,
}

impl Credentials {
    pub fn new(username: String, password: String, app_key: String) -> Self {
        Self {
            username,
            password,
            app_key,
        }
    }

    pub fn load() -> Result<Self> {
        let path = Self::credentials_path()?;

        if !path.exists() {
            anyhow::bail!(
                "No credentials found. Please run 'nsg login' first.\n\
                 Expected credentials at: {}",
                path.display()
            );
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read credentials from {}", path.display()))?;

        let creds: Credentials =
            serde_json::from_str(&content).context("Failed to parse credentials file")?;

        Ok(creds)
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).with_context(|| {
                format!(
                    "Failed to create config directory at {}",
                    config_dir.display()
                )
            })?;
        }

        let path = Self::credentials_path()?;
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize credentials")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write credentials to {}", path.display()))?;

        // Set file permissions to owner-only read/write
        Self::set_secure_permissions(&path)?;

        Ok(())
    }

    fn config_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home.join(CONFIG_DIR))
    }

    fn credentials_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join(CREDENTIALS_FILE))
    }

    pub fn credentials_location() -> String {
        Self::credentials_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| format!("~/{}/{}", CONFIG_DIR, CREDENTIALS_FILE))
    }

    /// Set file permissions to owner-only read/write (0600 on Unix, ACL on Windows)
    fn set_secure_permissions(path: &PathBuf) -> Result<()> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path)
                .context("Failed to get file metadata")?
                .permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms).context("Failed to set file permissions to 0600")?;
        }

        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;

            // On Windows, we need to use icacls or similar to set proper ACLs
            // Using a simpler approach: mark as hidden and system to discourage casual access
            let metadata = fs::metadata(path).context("Failed to get file metadata")?;

            // Set file attributes to hidden (not perfect, but better than nothing)
            let mut perms = metadata.permissions();
            perms.set_readonly(false); // Keep writable for the owner
            fs::set_permissions(path, perms).context("Failed to set file permissions")?;

            // Attempt to use icacls to set proper ACLs (owner-only access)
            // This is the proper way to secure files on Windows
            if let Err(e) = Self::set_windows_acl(path) {
                eprintln!(
                    "Warning: Could not set Windows ACL for credentials file: {}",
                    e
                );
                eprintln!("         File permissions may not be fully secure on Windows.");
                eprintln!("         Consider protecting your user account with a strong password.");
            }
        }

        Ok(())
    }

    #[cfg(windows)]
    fn set_windows_acl(path: &PathBuf) -> Result<()> {
        use std::process::Command;

        // Use icacls to:
        // 1. Disable inheritance (/inheritance:r)
        // 2. Grant current user full control (/grant:r %USERNAME%:F)
        let output = Command::new("icacls")
            .arg(path)
            .arg("/inheritance:r")
            .arg("/grant:r")
            .arg(format!(
                "{}:F",
                std::env::var("USERNAME").unwrap_or_else(|_| String::from("*S-1-5-32-544"))
            ))
            .output()
            .context("Failed to execute icacls command")?;

        if !output.status.success() {
            anyhow::bail!("icacls failed: {}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }
}
