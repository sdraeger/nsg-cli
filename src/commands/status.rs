use crate::client::NsgClient;
use crate::config::Credentials;
use anyhow::Result;
use clap::Args;
use colored::Colorize;

#[derive(Debug, Args)]
pub struct StatusCommand {
    #[arg(help = "Job URL or Job ID")]
    job: String,
}

impl StatusCommand {
    pub fn execute(self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = NsgClient::new(credentials)?;

        println!("{}", "NSG Job Status".bold().cyan());
        println!("{}", "=".repeat(80).cyan());
        println!();
        println!("{} Checking job status...", "→".cyan());
        println!("   Job: {}", self.job.bold());
        println!();

        let status = client.get_job_status(&self.job)?;

        println!("{} Job found", "✓".green().bold());
        println!();
        println!("{}", "Job Status Information".bold());
        println!("{}", "=".repeat(80));
        println!();
        println!("Job ID:       {}", status.job_id.cyan());

        let stage_icon = get_stage_icon(&status.job_stage);
        println!("Stage:        {} {}", stage_icon, status.job_stage.bold());

        if status.failed {
            println!("Failed:       {} YES", "✗".red().bold());
        }

        if let Some(date) = &status.date_submitted {
            println!("Submitted:    {}", format_timestamp(date));
        }

        if status.results_uri.is_some() {
            println!("Results:      {} Available", "✓".green());
        } else {
            println!("Results:      {} Not yet available", "⏳".yellow());
        }

        if !status.messages.is_empty() {
            println!();
            println!("{}", "Recent Messages:".bold());
            let recent = if status.messages.len() > 5 {
                &status.messages[status.messages.len() - 5..]
            } else {
                &status.messages[..]
            };

            for msg in recent {
                println!();
                println!(
                    "  [{}] {}",
                    msg.stage.cyan(),
                    msg.timestamp.as_deref().unwrap_or("")
                );
                if !msg.text.is_empty() {
                    let text = if msg.text.len() > 200 {
                        format!("{}...", &msg.text[..200])
                    } else {
                        msg.text.clone()
                    };
                    println!("    {}", text);
                }
            }
        }

        println!();
        println!("{}", "=".repeat(80));
        println!();

        print_next_action(&status.job_stage, &self.job);

        Ok(())
    }
}

fn get_stage_icon(stage: &str) -> &'static str {
    match stage {
        "COMPLETED" => "✓",
        "RUNNING" | "RUN" => "⟳",
        "QUEUE" | "SUBMITTED" => "⏳",
        "FAILED" => "✗",
        _ => "?",
    }
}

fn format_timestamp(ts: &str) -> String {
    use chrono::{DateTime, Utc};
    if let Ok(dt) = ts.parse::<DateTime<Utc>>() {
        dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    } else {
        ts.to_string()
    }
}

fn print_next_action(stage: &str, job_id: &str) {
    match stage {
        "COMPLETED" => {
            println!(
                "{} Job completed! You can now download results.",
                "✓".green().bold()
            );
            println!();
            println!("To download all results:");
            println!("  {}", format!("nsg download {}", job_id).cyan());
        }
        "FAILED" => {
            println!(
                "{} Job failed. Check messages above for error details.",
                "✗".red().bold()
            );
        }
        "QUEUE" | "SUBMITTED" => {
            println!("{} Job is queued. Check again later.", "⏳".yellow());
            println!();
            println!("To check status again:");
            println!("  {}", format!("nsg status {}", job_id).cyan());
        }
        "RUN" | "RUNNING" => {
            println!(
                "{} Job is running. Check back later for completion.",
                "⟳".yellow()
            );
            println!();
            println!("To check status again:");
            println!("  {}", format!("nsg status {}", job_id).cyan());
        }
        _ => {
            println!("{} Unknown job stage: {}", "?".yellow(), stage);
        }
    }
    println!();
}
