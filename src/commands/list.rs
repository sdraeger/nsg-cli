use crate::client::NsgClient;
use crate::config::Credentials;
use anyhow::Result;
use clap::Args;
use colored::Colorize;

#[derive(Debug, Args)]
pub struct ListCommand {
    #[arg(long, help = "Fetch detailed status for each job (slower)")]
    detailed: bool,

    #[arg(short, long, help = "Limit number of jobs to display")]
    limit: Option<usize>,

    #[arg(
        long,
        default_value = "20",
        help = "Show only the N most recent jobs (default: 20, use --recent 0 to show all)"
    )]
    recent: usize,

    #[arg(long, help = "Show all jobs (override default limit)")]
    all: bool,
}

impl ListCommand {
    pub fn execute(self) -> Result<()> {
        let credentials = Credentials::load()?;
        let client = NsgClient::new(credentials.clone())?;

        println!("{}", "NSG Job List".bold().cyan());
        println!("{}", "=".repeat(80).cyan());
        println!();
        println!(
            "{} Fetching jobs for user: {}",
            "→".cyan(),
            credentials.username.bold()
        );
        println!();

        let mut jobs = client.list_jobs()?;

        if jobs.is_empty() {
            println!("{}", "No jobs found".yellow());
            println!();
            println!("You can submit a test job with:");
            println!("  {}", "nsg submit <zip_file> --tool PY_EXPANSE".cyan());
            return Ok(());
        }

        let total_jobs = jobs.len();

        // Apply limit/recent filters
        if self.all {
            // Show all jobs, no filtering
        } else if let Some(limit) = self.limit {
            // Explicit limit takes precedence
            jobs.truncate(limit);
        } else if self.recent > 0 && jobs.len() > self.recent {
            // Default: show N most recent jobs
            jobs.drain(0..jobs.len() - self.recent);
        }

        let showing_jobs = jobs.len();

        if showing_jobs < total_jobs {
            println!(
                "Found {} job(s) total, showing {}",
                total_jobs.to_string().bold(),
                showing_jobs.to_string().bold()
            );
        } else {
            println!("Found {} job(s)", jobs.len().to_string().bold());
        }
        println!();
        println!("{}", "=".repeat(80));

        for (i, job) in jobs.iter().enumerate() {
            println!();
            println!("Job #{}", (i + 1).to_string().bold());
            println!("  ID:  {}", job.job_id.cyan());

            if self.detailed {
                println!("  {}", "Fetching details...".dimmed());
                match client.get_job_status(&job.url) {
                    Ok(status) => {
                        let stage_icon = get_stage_icon(&status.job_stage);
                        println!("  Status: {} {}", stage_icon, status.job_stage.bold());

                        if status.failed {
                            println!("  Failed: {} YES", "✗".red().bold());
                        }

                        if let Some(date) = &status.date_submitted {
                            println!("  Submitted: {}", format_timestamp(date));
                        }

                        if !status.messages.is_empty() {
                            if let Some(latest) = status.messages.last() {
                                println!(
                                    "  Latest: [{}] {}",
                                    latest.stage,
                                    truncate(&latest.text, 100)
                                );
                            }
                        }
                    }
                    Err(_) => {
                        println!("  Status: {} (failed to fetch)", "?".yellow());
                    }
                }
            } else {
                println!(
                    "  Status: {} (use --detailed for full status)",
                    "?".dimmed()
                );
            }

            println!("  URL: {}", job.url.dimmed());
            println!("{}", "=".repeat(80));
        }

        println!();
        println!("{}", "Commands:".bold());
        println!("  Check job status:    {}", "nsg status <JOB_ID>".cyan());
        println!("  Download results:    {}", "nsg download <JOB_ID>".cyan());

        if showing_jobs < total_jobs {
            println!();
            println!("{}", "Tip:".bold());
            println!("  Use {} to see all {} jobs", "--all".cyan(), total_jobs);
            println!("  Use {} to see detailed status", "--detailed".cyan());
            println!("  Use {} to limit results", "--limit N".cyan());
            println!("  Use {} to show N most recent jobs", "--recent N".cyan());
        }
        println!();

        Ok(())
    }
}

fn get_stage_icon(stage: &str) -> String {
    match stage {
        "COMPLETED" => "✓".green().bold().to_string(),
        "RUNNING" | "RUN" => "⟳".yellow().bold().to_string(),
        "QUEUE" | "SUBMITTED" => "⏳".cyan().to_string(),
        "FAILED" => "✗".red().bold().to_string(),
        _ => "?".dimmed().to_string(),
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

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
