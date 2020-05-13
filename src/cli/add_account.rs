use super::CLICommand;
use crate::subcommands::run_add_account;
use crate::{tag_parser, tags_validator, Tag};
use clap::{App, Arg, ArgMatches, SubCommand};

pub struct AddAccountCommand {}

#[async_trait::async_trait]
impl CLICommand for AddAccountCommand {
    fn subcommand(&self) -> App<'_, '_> {
        SubCommand::with_name("add_account")
            .about("Add ability to deploy into a new account")
            .arg(
                Arg::with_name("profile")
                    .long("profile")
                    .help("AWS profile to use when adding the new account")
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
        "add_account"
    }

    async fn run_fn(&self, matches: &ArgMatches<'_>) {
        let profile = matches.value_of("profile").unwrap().to_owned();

        let tags = matches
            .values_of("tags")
            .map(|values| values.map(parse_tags).flatten().collect())
            .unwrap_or(vec![]);

        let result = run_add_account(profile, tags).await;
        match result {
            Ok(_) => {
                println!("Successfully added account");
            }
            Err(err) => {
                println!("Failed to add account with error: {:#?}", err);
            }
        }
    }

    fn create() -> Self {
        AddAccountCommand {}
    }
}

fn parse_tags(data: &str) -> Vec<Tag> {
    tag_parser(data.to_owned()).unwrap_or(vec![])
}
