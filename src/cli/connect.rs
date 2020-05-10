use clap::{App, ArgMatches, SubCommand};

use super::CLICommand;
use crate::run_connect;

pub struct ConnectCommand {}

#[async_trait::async_trait]
impl CLICommand for ConnectCommand {
    fn subcommand(&self) -> App<'_, '_> {
        SubCommand::with_name("connect")
            .about("Create command which will ssh into box")
            .usage("$(builder connect)")
    }

    fn command_name(&self) -> &'static str {
        "connect"
    }

    async fn run_fn(&self, _matches: &ArgMatches<'_>) {
        println!("{}", run_connect());
    }

    fn create() -> Self {
        ConnectCommand {}
    }
}
