use clap::{App, ArgMatches, SubCommand};

use super::CLICommand;
use crate::uninstall;

pub struct UninstallCommand {}

#[async_trait::async_trait]
impl CLICommand for UninstallCommand {
    fn subcommand(&self) -> App<'_, '_> {
        SubCommand::with_name("uninstall").about("Clean up instance & SSH key")
    }

    fn command_name(&self) -> &'static str {
        "uninstall"
    }

    async fn run_fn(&self, _matches: &ArgMatches<'_>) {
        let result = uninstall().await;
        match result {
            Ok(()) => {
                println!("Successfully uninstalled resources");
            }
            Err(err) => {
                println!("Failed to uninstall with error: {:#?}", err);
            }
        }
    }

    fn create() -> Self {
        UninstallCommand {}
    }
}
