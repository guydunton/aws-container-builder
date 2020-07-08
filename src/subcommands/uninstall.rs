use crate::{CfnClient, Config, EC2Client};
use futures::future::join_all;

#[derive(Debug)]
pub enum UninstallError {
    CouldNotFindConfig,
    DeleteStackFailed,
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

    let cfn_client = CfnClient::new(config.get_base_profile());
    let deleted_stack = cfn_client
        .delete_stack("container-builder".to_owned())
        .await;

    if !deleted_stack {
        return Err(UninstallError::DeleteStackFailed);
    }

    // Delete the stacks from all the other accounts
    let results: Vec<UninstallError> = join_all(config.get_account_numbers().into_iter().map(
        |account_id| async {
            let cfn_client_sub = CfnClient::new(
                config
                    .get_account_profile(account_id)
                    .expect("Failed to get profile for account id"),
            );

            cfn_client_sub
                .delete_stack("container-builder-role".to_owned())
                .await
        },
    ))
    .await
    .into_iter()
    .filter_map(|res: bool| {
        if !res {
            Some(UninstallError::DeleteStackFailed)
        } else {
            None
        }
    })
    .collect();

    if results.len() > 0 {
        return Err(UninstallError::DeleteStackFailed);
    }

    // delete the ssh key
    let ec2_client = EC2Client::new(config.get_base_profile());

    let deleted_key = ec2_client
        .delete_key_pair("ContainerBuilderKey".to_owned())
        .await;
    if !deleted_key {
        return Err(UninstallError::DeleteSSHKeyFailed(
            "Failed to delete SSH key".to_owned(),
        ));
    }

    // Delete the local ssh key and properties file
    std::fs::remove_file(&props_file_path).map_err(|_| UninstallError::CouldNotDeleteLocalFiles)?;
    std::fs::remove_file(&working_dir.join("ContainerBuilderKey.pem"))
        .map_err(|_| UninstallError::CouldNotDeleteLocalFiles)?;

    Ok(())
}
