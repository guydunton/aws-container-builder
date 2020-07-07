use clap::{App, ArgMatches, SubCommand};

use super::CLICommand;
use crate::subcommands::run_stop;

pub struct StopCommand {}

impl StopCommand {
    pub fn new() -> Self {
        StopCommand {}
    }
}

#[async_trait::async_trait]
impl CLICommand for StopCommand {
    fn subcommand(&self) -> App<'_, '_> {
        SubCommand::with_name("stop")
            .about("Stop the running EC2 instance. It can be restarted using the start command")
    }

    fn command_name(&self) -> &'static str {
        "stop"
    }

    async fn run_fn(&self, _matches: &ArgMatches<'_>) {
        let result = run_stop().await;
        match result {
            Ok(_) => println!(
                "Successfully stopped instance. Start instance again using command:
    builder start"
            ),
            Err(err) => {
                println!("Failed to stop instance with error: {}", err);
            }
        }
    }
}
