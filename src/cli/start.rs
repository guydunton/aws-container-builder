use clap::{App, ArgMatches, SubCommand};

use super::CLICommand;
use crate::subcommands::run_start;

pub struct StartCommand {}

impl StartCommand {
    pub fn new() -> Self {
        StartCommand {}
    }
}

#[async_trait::async_trait]
impl CLICommand for StartCommand {
    fn subcommand(&self) -> App<'_, '_> {
        SubCommand::with_name("start").about("Start the underlying EC2 instance. ")
    }

    fn command_name(&self) -> &'static str {
        "start"
    }

    async fn run_fn(&self, _matches: &ArgMatches<'_>) {
        let result = run_start().await;

        match result {
            Ok(_) => {
                println!("Successfully started instance");
            }
            Err(err) => {
                println!("Failed to start instance with error: {:?}", err);
            }
        }
    }
}
