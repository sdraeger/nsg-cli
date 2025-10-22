use clap::{Parser, Subcommand};
use colored::Colorize;
use nsg_cli::commands::*;

#[derive(Debug, Parser)]
#[command(
    name = "nsg",
    version,
    about = "CLI tool for the Neuroscience Gateway (NSG) BRAIN Initiative",
    long_about = "A command-line interface for interacting with the Neuroscience Gateway (NSG) \
                  REST API. Submit jobs, check status, and download results from NSG HPC clusters."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "Login and save NSG credentials")]
    Login(LoginCommand),

    #[command(about = "List all jobs for the authenticated user")]
    List(ListCommand),

    #[command(about = "Check the status of a specific job")]
    Status(StatusCommand),

    #[command(about = "Submit a new job to NSG")]
    Submit(SubmitCommand),

    #[command(about = "Download results from a completed job")]
    Download(DownloadCommand),
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Login(cmd) => cmd.execute(),
        Commands::List(cmd) => cmd.execute(),
        Commands::Status(cmd) => cmd.execute(),
        Commands::Submit(cmd) => cmd.execute(),
        Commands::Download(cmd) => cmd.execute(),
    };

    if let Err(e) = result {
        eprintln!();
        eprintln!("{} {}", "Error:".red().bold(), e);

        if let Some(source) = e.source() {
            eprintln!();
            eprintln!("{} {}", "Caused by:".red(), source);
        }

        eprintln!();
        std::process::exit(1);
    }
}
