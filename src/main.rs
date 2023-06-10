mod sync;
mod todo;
mod util;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Move all undone tasks to today's file
    #[arg(short, long)]
    undone: bool,

    /// Open file with date (e.g. today, tom, thu, 2021-01-01, 12-01, 07)
    ///
    /// Expects YYYY-MM-DD, MM-DD or DD
    /// Will default to the current year and month if only DD is provided etc
    /// Alternatively you can pass a weekday (mon, tue, wed, thu, fri, sat or sun) from the last week
    #[arg(index = 1)]
    date: Option<String>,

    /// Sync doto files with server
    #[clap(short, long)]
    sync: bool,

    // Login for syncing
    // #[clap(short, long)]
    // login: bool,
    // Sync all todo files
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Login,
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Login) => sync::auth::login(),
        _ => {
            // default behaviour
            if cli.sync {
                println!("Syncing...");
            } else if cli.undone {
                todo::move_undone();
            } else {
                match cli.date {
                    Some(date) => todo::open_date(date),
                    None => todo::open_week(),
                }
            }
        }
    }
}
