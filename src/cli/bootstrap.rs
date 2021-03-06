use super::CLICommand;
use crate::{run_bootstrap, tag_parser, tags_validator, Tag};
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct BootstrapCommand {}

impl BootstrapCommand {
    pub fn new() -> Self {
        BootstrapCommand {}
    }
}

#[async_trait::async_trait]
impl CLICommand for BootstrapCommand {
    fn subcommand(&self) -> App<'_, '_> {
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
            )
    }

    fn command_name(&self) -> &'static str {
        "bootstrap"
    }

    async fn run_fn(&self, matches: &ArgMatches<'_>) {
        let profile = matches.value_of("profile").unwrap();

        let tags = matches
            .values_of("tags")
            .map(|values| values.map(parse_tags).flatten().collect())
            .unwrap_or(vec![]);

        let result = run_bootstrap(profile.to_owned(), tags).await;
        println!("Finished bootstrap with result: {:?}", result);
    }
}

fn parse_tags(data: &str) -> Vec<Tag> {
    tag_parser(data.to_owned()).unwrap_or(vec![])
}
