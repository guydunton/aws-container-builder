use crate::get_current_account_no;
use crate::{CfnClient, Config, SimpleParameter, Tag};

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

    // Work out the new AccountRoles parameter based off the account and the base account
    let mut current_accounts = config.get_account_numbers();
    current_accounts.push(new_account_id.clone());
    current_accounts.push(base_account_id.clone());
    let accounts_parameters_value = current_accounts
        .iter()
        .map(|id| format!("arn:aws:iam::{}:role/ContainerBuilderPushRole", id))
        .collect::<Vec<String>>()
        .join(",");

    let cfn_client = CfnClient::new(config.get_base_profile());

    // Redeploy the base cloudformation stack with extra permissions
    let did_update_stack = cfn_client.update_stack(accounts_parameters_value).await;

    if !did_update_stack {
        return Err(AddAccountError::FailedUpdateStack("".to_owned()));
    }

    // Deploy the role stack in the new account
    let new_stacktemplate = std::fs::read_to_string("resources/role-cfn.yml")
        .map_err(|_| AddAccountError::FailedCreateNewStack)?;

    // Create cloudformation client
    let new_cfn_client = CfnClient::new(new_account_profile.clone());
    new_cfn_client
        .deploy_stack(
            "container-builder-role".to_owned(),
            new_stacktemplate,
            &vec![SimpleParameter::new(
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
