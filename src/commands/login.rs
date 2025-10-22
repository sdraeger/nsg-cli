use anyhow::Result;
use clap::Args;
use colored::Colorize;
use std::io::{self, Write};
use crate::client::NsgClient;
use crate::config::Credentials;

#[derive(Debug, Args)]
pub struct LoginCommand {
    #[arg(short, long, help = "NSG username")]
    username: Option<String>,

    #[arg(short, long, help = "NSG password")]
    password: Option<String>,

    #[arg(short, long, help = "NSG application key")]
    app_key: Option<String>,

    #[arg(long, help = "Skip connection test")]
    no_verify: bool,
}

impl LoginCommand {
    pub fn execute(self) -> Result<()> {
        println!("{}", "NSG Login".bold().cyan());
        println!("{}", "=".repeat(60).cyan());
        println!();

        let username = self.get_or_prompt_username()?;
        let password = self.get_or_prompt_password()?;
        let app_key = self.get_or_prompt_app_key()?;

        println!();
        println!("{} Saving credentials...", "→".cyan());

        let credentials = Credentials::new(username, password, app_key);

        if !self.no_verify {
            println!("{} Testing connection to NSG...", "→".cyan());
            let client = NsgClient::new(credentials.clone())?;

            match client.test_connection() {
                Ok(_) => {
                    println!("{} Connection successful!", "✓".green().bold());
                }
                Err(e) => {
                    eprintln!();
                    eprintln!("{} {}", "✗".red().bold(), "Authentication failed!".red());
                    eprintln!();
                    eprintln!("Error: {}", e);
                    eprintln!();
                    eprintln!("Please check your credentials:");
                    eprintln!("  1. Username and password are correct");
                    eprintln!("  2. Application key is valid");
                    eprintln!("  3. Your NSG account is active");
                    eprintln!();
                    eprintln!("Get credentials at: {}", "https://www.nsgportal.org/".cyan());
                    anyhow::bail!("Login failed");
                }
            }
        }

        credentials.save()?;

        println!();
        println!("{}", "=".repeat(60).green());
        println!("{} {}", "✓".green().bold(), "Login successful!".green().bold());
        println!("{}", "=".repeat(60).green());
        println!();
        println!("Credentials saved to: {}", Credentials::credentials_location().cyan());
        println!();
        println!("You can now use:");
        println!("  {} - List your NSG jobs", "nsg list".cyan());
        println!("  {} - Check job status", "nsg status <job_id>".cyan());
        println!("  {} - Submit a new job", "nsg submit <zip_file> --tool <tool>".cyan());
        println!("  {} - Download job results", "nsg download <job_id>".cyan());
        println!();

        Ok(())
    }

    fn get_or_prompt_username(&self) -> Result<String> {
        if let Some(username) = &self.username {
            return Ok(username.clone());
        }

        print!("NSG Username: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let username = input.trim().to_string();

        if username.is_empty() {
            anyhow::bail!("Username cannot be empty");
        }

        Ok(username)
    }

    fn get_or_prompt_password(&self) -> Result<String> {
        if let Some(password) = &self.password {
            return Ok(password.clone());
        }

        print!("NSG Password: ");
        io::stdout().flush()?;
        let password = rpassword::read_password()?;

        if password.is_empty() {
            anyhow::bail!("Password cannot be empty");
        }

        Ok(password)
    }

    fn get_or_prompt_app_key(&self) -> Result<String> {
        if let Some(app_key) = &self.app_key {
            return Ok(app_key.clone());
        }

        print!("NSG Application Key: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let app_key = input.trim().to_string();

        if app_key.is_empty() {
            anyhow::bail!("Application key cannot be empty");
        }

        Ok(app_key)
    }
}
