use clap::{Parser, Subcommand};
// use clap_complete::{generate, Generator, Shell};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a default config file
    Init {},
    /// Check that the config file is valid
    Validate {},
    /// Run the input file (or config file) and pretty print the result
    Eval {},
    /// Format the config file nicely
    Format {},
    /// Run the config and apply it to the system
    Apply {},
    /// Go back to the state of the system before running Apply
    Revert {},
    /// Pull the latest version of config from user git repo
    Pull {},
    /// Push the current version of config to the user git repo
    Push {},
    /// Update this program to the newest version
    Upgrade {},
    /// Open the config in the user's editor
    Edit {},
    /// Remove this from the user's system
    Uninstall {},
    /// Generate shell completion code
    Completions {},
    /// Run git in the config directory
    Git {},
    /// cd into the config directory
    Cd {},
}