use crate::Config;
use rusoto_cloudformation::{CloudFormation, CloudFormationClient, DeleteStackInput};
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_ec2::{DeleteKeyPairRequest, Ec2, Ec2Client};

#[derive(Debug)]
pub enum UninstallError {
    CouldNotFindConfig,
    DeleteStackFailed(String),
    DeleteSSHKeyFailed(String),
    CouldNotDeleteLocalFiles,
}

pub async fn uninstall() -> Result<(), UninstallError> {
    // Load the config
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");
    let props_file_path = working_dir.join("properties.yml");
    let config =
        Config::read_from_file(&props_file_path).ok_or(UninstallError::CouldNotFindConfig)?;

    let profile_provider = ProfileProvider::with_configuration(
        home_dir.join(".aws/credentials"),
        config.get_base_profile(),
    );

    // Create cloudformation client
    let cfn_client = CloudFormationClient::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    // Try to delete the cloudformation stack
    cfn_client
        .delete_stack(DeleteStackInput {
            stack_name: "container-builder".to_owned(),
            client_request_token: None,
            retain_resources: None,
            role_arn: None,
        })
        .await
        .map_err(|err| UninstallError::DeleteStackFailed(err.to_string()))?;

    // delete the ssh key
    let ec2_client = Ec2Client::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    ec2_client
        .delete_key_pair(DeleteKeyPairRequest {
            key_name: "ContainerBuilderKey".to_owned(),
            dry_run: Some(false),
        })
        .await
        .map_err(|err| UninstallError::DeleteSSHKeyFailed(err.to_string()))?;

    // Delete the local ssh key and properties file
    std::fs::remove_file(&props_file_path).map_err(|_| UninstallError::CouldNotDeleteLocalFiles)?;
    std::fs::remove_file(&working_dir.join("ContainerBuilderKey.pem"))
        .map_err(|_| UninstallError::CouldNotDeleteLocalFiles)?;

    Ok(())
}
