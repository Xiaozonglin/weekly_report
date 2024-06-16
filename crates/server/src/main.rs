use std::process::exit;

use clap::{Parser, Subcommand};
use colored::Colorize;
use wr_server::{greet, up};

/// Clap arg definition.
#[derive(Parser, Debug)]
#[command(
    author = "Reverier-Xu <reverier.xu@xdsec.club>",
    version,
    about = "XDSEC Weekly Report backend server.",
    long_about = r#"
XDSEC Weekly Report backend server

THE CONTENTS OF THIS PROJECT ARE PROPRIETARY AND CONFIDENTIAL.
UNAUTHORIZED COPYING, TRANSFERRING OR REPRODUCTION OF THE CONTENTS OF THIS PROJECT,
VIA ANY MEDIUM IS STRICTLY PROHIBITED.

If you have any problems, please contact developer <reverier.xu@xdsec.club>.
    "#
)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

/// Clap subcommands.
#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the server.
    Up,
}

/// Server entry.
#[tokio::main]
async fn main() {
    greet();
    // Parse command line arguments
    let args: Args = Args::parse();
    match match args.command {
        Some(Commands::Up) => up().await,
        None => up().await,
    } {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}: {e}", "Failed to start server".red().bold());
            eprintln!("Please check your configuration file and try again.");
            eprintln!("If you are still suffering from this problem and don't know how to fix it, please contact developer <reverier.xu@xdsec.club>.");
            exit(1)
        }
    }
}
