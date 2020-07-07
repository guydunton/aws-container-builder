mod add_account;
mod bootstrap;
mod connect;
mod ship;
mod start;
mod stop;
mod uninstall;

pub use add_account::AddAccountCommand;
pub use bootstrap::BootstrapCommand;
pub use connect::ConnectCommand;
pub use ship::ShipCommand;
pub use start::StartCommand;
pub use stop::StopCommand;
pub use uninstall::UninstallCommand;

use clap::{App, ArgMatches};

#[async_trait::async_trait]
pub trait CLICommand {
    fn subcommand(&self) -> App<'_, '_>;
    fn command_name(&self) -> &'static str;
    async fn run_fn(&self, matches: &ArgMatches<'_>);
}

pub async fn run_if_called<C: CLICommand>(cli_command: &C, global_matches: &ArgMatches<'_>) {
    if let Some(matches) = global_matches.subcommand_matches(cli_command.command_name()) {
        cli_command.run_fn(matches).await;
    }
}
