use super::CLICommand;
use crate::subcommands::ship;
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct ShipCommand {}

#[async_trait::async_trait]
impl CLICommand for ShipCommand {
    fn subcommand(&self) -> App<'_, '_> {
        let ship_help = "Zip up the current directory, send to instance, build docker image and push to container registry";
        SubCommand::with_name("ship")
            .about(ship_help)
            .arg(
                Arg::with_name("path")
                    .long("path")
                    .short("p")
                    .help("Path to the directory to build")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("registry")
                    .long("registry")
                    .short("r")
                    .help("The registry URI where we want to push the container")
                    .takes_value(true)
                    .required(true),
            )
            .arg(
                Arg::with_name("tag")
                    .long("tag")
                    .short("t")
                    .help("Docker tag to apply to the build")
                    .default_value("latest"),
            )
            .arg(
                Arg::with_name("build_args")
                    .last(true)
                    .required(false)
                    .multiple(true)
                    .help("Additional arguments supplied to docker build"),
            )
    }

    fn command_name(&self) -> &'static str {
        "ship"
    }

    async fn run_fn(&self, matches: &ArgMatches<'_>) {
        let path = matches.value_of("path").unwrap().to_owned();
        let registry_uri = matches.value_of("registry").unwrap().to_owned();
        let additional_args: Option<Vec<String>> = matches
            .values_of("build_args")
            .map(|values| values.map(|value| value.to_owned()).collect());
        let tag = matches.value_of("tag").unwrap().to_owned();

        let result = ship(path, registry_uri, additional_args, tag);
        // ship subcommand
        match result {
            Ok(()) => {
                println!("Ship was successful");
            }
            Err(err) => {
                println!("ship failed with error: {:#?}", err);
            }
        }
    }

    fn create() -> Self {
        ShipCommand {}
    }
}
