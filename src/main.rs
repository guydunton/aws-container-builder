use bootstrap::run_bootstrap;
use clap::{App, AppSettings, Arg, SubCommand};

mod bootstrap;
mod cfn_deploy;
mod config;
mod connect;
mod docker_ignore;
mod get_stack_ip;
mod ship;
mod ssh_client;
mod tag;
mod test;

pub use config::Config;
pub use tag::Tag;
use tag::{tag_parser, tags_validator};

#[tokio::main]
async fn main() {
    let ship_help = "Zip up the current directory, send to instance, build docker image and push to container registry";

    let matches = App::new("builder")
        .name("AWS container builder")
        .version("0.2")
        .setting(AppSettings::VersionlessSubcommands)
        .about("This program uses EC2 instances to build containers to save bandwidth")
        .subcommand(
            SubCommand::with_name("bootstrap")
                .about("Setup EC2 environment")
                .arg(
                    Arg::with_name("profile")
                        .long("profile")
                        .help("AWS profile to use when setting up environment")
                        .env("AWS_PROFILE")
                        .takes_value(true)
                        .hide_env_values(true)
                        .required(true),
                )
                .arg(
                    Arg::with_name("tags")
                        .long("tags")
                        .help("Additional tags to apply to AWS resources")
                        .long_help(
                            "Additional tags to apply to AWS resources. Usage should be: 
    --tags Key=Value Key2=Value2 OR --tags Key=Value,Key2=Value2",
                        )
                        .required(false)
                        .takes_value(true)
                        .multiple(true) // NOTE: Don't put positional args after this
                        .validator(tags_validator),
                ),
        )
        .subcommand(
            SubCommand::with_name("connect")
                .about("Create command which will ssh into box")
                .usage("$(builder connect)"),
        )
        .subcommand(
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
                    Arg::with_name("build_args")
                        .last(true)
                        .required(false)
                        .multiple(true)
                        .help("Additional arguments supplied to docker build"),
                ),
        )
        .get_matches();

    // Handle subcommands
    if let Some(matches) = matches.subcommand_matches("bootstrap") {
        println!("called bootstrap");
        let profile = matches.value_of("profile").unwrap();

        let tags = matches
            .values_of("tags")
            .map(|values| values.map(parse_tags).flatten().collect())
            .unwrap_or(vec![]);

        let result = run_bootstrap(profile.to_owned(), tags).await;
        println!("Finished bootstrap with result: {:?}", result);
    } else if let Some(_) = matches.subcommand_matches("connect") {
        // Print the command
        println!("{}", connect::connect());
    } else if let Some(matches) = matches.subcommand_matches("ship") {
        let path = matches.value_of("path").unwrap().to_owned();
        let registry_uri = matches.value_of("registry").unwrap().to_owned();
        let additional_args: Option<Vec<String>> = matches
            .values_of("build_args")
            .map(|values| values.map(|value| value.to_owned()).collect());

        let result = ship::ship(path, registry_uri, additional_args);
        // ship subcommand
        println!("ship command: {:?}", result);
    }
}

fn parse_tags(data: &str) -> Vec<Tag> {
    tag_parser(data.to_owned()).unwrap_or(vec![])
}
