use crate::client::NsgClient;
use crate::config::Credentials;
use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

#[derive(Debug, Args)]
pub struct SubmitCommand {
    #[arg(help = "Path to ZIP file containing job data")]
    zip_file: PathBuf,

    #[arg(short, long, default_value = "PY_EXPANSE", help = "NSG tool to use")]
    tool: String,

    #[arg(long, help = "Don't wait for job submission confirmation")]
    no_wait: bool,
}

impl SubmitCommand {
    pub fn execute(self) -> Result<()> {
        if !self.zip_file.exists() {
            anyhow::bail!("ZIP file not found: {}", self.zip_file.display());
        }

        if !self.zip_file.extension().map_or(false, |ext| ext == "zip") {
            eprintln!("{} File does not have .zip extension", "⚠".yellow());
            eprintln!("   Continuing anyway...");
            eprintln!();
        }

        let credentials = Credentials::load()?;
        let client = NsgClient::new(credentials.clone())?;

        println!("{}", "NSG Job Submission".bold().cyan());
        println!("{}", "=".repeat(80).cyan());
        println!();
        println!("Tool:     {}", self.tool.bold());
        println!("User:     {}", credentials.username.cyan());
        println!("File:     {}", self.zip_file.display().to_string().cyan());
        println!(
            "Size:     {} bytes",
            format_size(std::fs::metadata(&self.zip_file)?.len())
        );
        println!();

        println!("{} Submitting job to NSG...", "→".yellow().bold());

        let status = client
            .submit_job(&self.zip_file, &self.tool)
            .context("Failed to submit job")?;

        println!();
        println!("{}", "=".repeat(80).green());
        println!("{} Job submitted successfully!", "✓".green().bold());
        println!("{}", "=".repeat(80).green());
        println!();
        println!("Job ID:   {}", status.job_id.cyan().bold());
        println!("Stage:    {}", status.job_stage.bold());
        println!("URL:      {}", status.self_uri.dimmed());

        if let Some(date) = &status.date_submitted {
            println!("Submitted: {}", date);
        }

        println!();
        println!("{}", "Next Steps:".bold());
        println!("  1. Check job status:");
        println!("     {}", format!("nsg status {}", status.job_id).cyan());
        println!();
        println!("  2. When completed, download results:");
        println!("     {}", format!("nsg download {}", status.job_id).cyan());
        println!();
        println!("  3. View all jobs:");
        println!("     {}", "nsg list".cyan());
        println!();
        println!("{}", "NSG Portal:".bold());
        println!("  {}", "https://www.nsgportal.org/".cyan());
        println!();

        Ok(())
    }
}

fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
