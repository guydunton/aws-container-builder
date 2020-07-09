use clap::{App, AppSettings};

mod cli;
mod entities;
mod subcommands;

use cli::CLICommand;
pub use entities::*;
use subcommands::*;

#[tokio::main]
async fn main() {
    let ship_subcommand = cli::ShipCommand::new();
    let bootstrap_subcommand = cli::BootstrapCommand::new();
    let uninstall_subcommand = cli::UninstallCommand::new();
    let connect_subcommand = cli::ConnectCommand::new();
    let add_account_subcommand = cli::AddAccountCommand::new();
    let start_subcommand = cli::StartCommand::new();
    let stop_subcommand = cli::StopCommand::new();

    let matches = App::new("builder")
        .name("AWS container builder")
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::VersionlessSubcommands)
        .about("This program uses EC2 instances to build containers to save bandwidth")
        .subcommand(bootstrap_subcommand.subcommand())
        .subcommand(connect_subcommand.subcommand())
        .subcommand(uninstall_subcommand.subcommand())
        .subcommand(ship_subcommand.subcommand())
        .subcommand(add_account_subcommand.subcommand())
        .subcommand(start_subcommand.subcommand())
        .subcommand(stop_subcommand.subcommand())
        .get_matches();

    // Handle subcommands
    cli::run_if_called(&connect_subcommand, &matches).await;
    cli::run_if_called(&ship_subcommand, &matches).await;
    cli::run_if_called(&bootstrap_subcommand, &matches).await;
    cli::run_if_called(&uninstall_subcommand, &matches).await;
    cli::run_if_called(&add_account_subcommand, &matches).await;
    cli::run_if_called(&start_subcommand, &matches).await;
    cli::run_if_called(&stop_subcommand, &matches).await;
}
