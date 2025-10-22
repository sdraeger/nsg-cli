use anyhow::Result;
use clap::Args;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use crate::client::NsgClient;
use crate::config::Credentials;

#[derive(Debug, Args)]
pub struct DownloadCommand {
    #[arg(help = "Job URL or Job ID")]
    job: String,

    #[arg(short, long, default_value = "./nsg_results", help = "Output directory")]
    output: PathBuf,
}

impl DownloadCommand {
    pub fn execute(self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = NsgClient::new(credentials)?;

        println!("{}", "NSG Results Downloader".bold().cyan());
        println!("{}", "=".repeat(80).cyan());
        println!();
        println!("{} Checking job status...", "→".cyan());
        println!("   Job: {}", self.job.bold());
        println!();

        let status = client.get_job_status(&self.job)?;

        println!("Job ID:       {}", status.job_id.cyan());
        println!("Stage:        {}", status.job_stage.bold());

        if status.job_stage != "COMPLETED" {
            println!();
            println!("{} Job is not completed yet", "⚠".yellow().bold());
            println!("   Current stage: {}", status.job_stage.bold());
            println!();
            println!("Results may not be available. Continue anyway? [y/N] ");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }

        println!();
        println!("{} Output directory: {}", "→".cyan(), self.output.display().to_string().bold());
        println!();

        if self.output.exists() && std::fs::read_dir(&self.output)?.next().is_some() {
            println!("{} Directory already exists and is not empty", "⚠".yellow());
            println!("   Files may be overwritten. Continue? [y/N] ");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            if !input.trim().eq_ignore_ascii_case("y") {
                println!("Cancelled.");
                return Ok(());
            }
        }

        println!("{} Downloading output files...", "→".yellow().bold());
        println!();

        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Fetching file list...");

        let downloaded = client.download_results(&self.job, &self.output)?;

        pb.finish_and_clear();

        if downloaded.is_empty() {
            println!("{} No output files found", "⚠".yellow());
            println!();
            println!("This could mean:");
            println!("  1. Job hasn't produced output files yet");
            println!("  2. Job failed without creating outputs");
            println!("  3. Check stderr.txt and stdout.txt for details");
            return Ok(());
        }

        println!("{} Downloaded {} file(s):", "✓".green().bold(), downloaded.len());
        println!();

        let mut total_size = 0u64;
        for file in &downloaded {
            total_size += file.size;
            println!("  {} {} ({})", "✓".green(), file.filename.cyan(), format_size(file.size));
        }

        println!();
        println!("{}", "=".repeat(80).green());
        println!("{} Download complete!", "✓".green().bold());
        println!("{}", "=".repeat(80).green());
        println!();
        println!("Location:     {}", self.output.display().to_string().cyan());
        println!("Files:        {}", downloaded.len());
        println!("Total size:   {}", format_size(total_size));
        println!();

        if downloaded.iter().any(|f| f.filename == "dda_results.json") {
            println!("{} DDA results found!", "✓".green());
            println!();
            println!("View results:");
            let path = self.output.join("dda_results.json");
            println!("  cat {} | jq .", path.display());
        }

        if downloaded.iter().any(|f| f.filename == "stderr.txt") {
            println!();
            println!("{} stderr.txt exists - check for errors:", "⚠".yellow());
            let path = self.output.join("stderr.txt");
            println!("  cat {}", path.display());
        }

        if downloaded.iter().any(|f| f.filename == "stdout.txt") {
            println!();
            println!("stdout.txt exists:");
            let path = self.output.join("stdout.txt");
            println!("  cat {}", path.display());
        }

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
