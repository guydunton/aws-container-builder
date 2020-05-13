use crate::cfn_deploy::deploy_stack;
use crate::get_current_account_no;
use crate::{Config, Tag};
use rusoto_cloudformation::{CloudFormation, CloudFormationClient, Parameter, UpdateStackInput};
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};

#[derive(Debug)]
pub enum AddAccountError {
    FailedToLoadConfig,
    FailedGetCurrentAccountNo(String),
    FailedUpdateStack(String),
    FailedCreateNewStack,
    FailedToUpdateConfig,
}

pub async fn run_add_account(
    new_account_profile: String,
    tags: Vec<Tag>,
) -> Result<(), AddAccountError> {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    // Load the config
    let mut config = Config::read_from_file(&working_dir.join("properties.yml"))
        .ok_or(AddAccountError::FailedToLoadConfig)?;

    // Find the base account id
    let base_account_id = get_current_account_no(config.get_base_profile())
        .await
        .map_err(|err| AddAccountError::FailedGetCurrentAccountNo(err))?;

    let new_account_id = get_current_account_no(new_account_profile.clone())
        .await
        .map_err(|err| AddAccountError::FailedGetCurrentAccountNo(err))?;

    let base_profile_provider = ProfileProvider::with_configuration(
        home_dir.join(".aws/credentials"),
        config.get_base_profile().clone(),
    );

    // Create cloudformation client
    let cfn_client = CloudFormationClient::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        base_profile_provider.clone(),
        Region::UsEast1,
    );

    // Work out the new AccountRoles parameter based off the account and the base account
    let mut current_accounts = config.get_account_numbers();
    current_accounts.push(new_account_id.clone());
    current_accounts.push(base_account_id.clone());
    let accounts_parameters_value = current_accounts
        .iter()
        .map(|id| format!("arn:aws:iam::{}:role/ContainerBuilderPushRole", id))
        .collect::<Vec<String>>()
        .join(",");

    // Redeploy the base cloudformation stack with extra permissions
    cfn_client
        .update_stack(UpdateStackInput {
            capabilities: Some(vec!["CAPABILITY_NAMED_IAM".to_owned()]),
            stack_name: "container-builder".to_owned(),
            parameters: Some(vec![
                Parameter {
                    parameter_key: Some("AmiId".to_owned()),
                    use_previous_value: Some(true),
                    ..Parameter::default()
                },
                Parameter {
                    parameter_key: Some("SSHKeyName".to_owned()),
                    use_previous_value: Some(true),
                    ..Parameter::default()
                },
                Parameter {
                    parameter_key: Some("AccountRoles".to_owned()),
                    parameter_value: Some(accounts_parameters_value),
                    ..Parameter::default()
                },
            ]),
            use_previous_template: Some(true),
            ..UpdateStackInput::default()
        })
        .await
        .map_err(|err| AddAccountError::FailedUpdateStack(err.to_string()))?;

    // Deploy the role stack in the new account
    let new_stacktemplate = std::fs::read_to_string("resources/role-cfn.yml")
        .map_err(|_| AddAccountError::FailedCreateNewStack)?;

    let new_profile_provider = ProfileProvider::with_configuration(
        home_dir.join(".aws/credentials"),
        new_account_profile.clone(),
    );

    // Create cloudformation client
    let new_cfn_client = CloudFormationClient::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        new_profile_provider,
        Region::UsEast1,
    );

    deploy_stack(
        &new_cfn_client,
        "container-builder-role".to_owned(),
        new_stacktemplate,
        &vec![crate::cfn_deploy::Parameter::new(
            "RootAccount".to_owned(),
            base_account_id.clone(),
        )],
        &tags,
        7 * 60,
    )
    .await
    .map_err(|_| AddAccountError::FailedCreateNewStack)?;

    // Add to config
    config.add_account(new_account_profile, new_account_id);
    config
        .write_to_file(&working_dir.join("properties.yml"))
        .map_err(|_| AddAccountError::FailedToUpdateConfig)?;

    Ok(())
}
