use crate::Config;
use crate::{CfnClient, EC2Client};
use std::{thread::sleep, time::Duration};

#[derive(Debug)]
pub enum StartError {
    CouldNotFindConfig,
    InstanceNotFound,
    FailedToStart,
    DescribeInstanceFailed,
    FailedSaveConfig,
}

pub async fn run_start() -> Result<(), StartError> {
    // Get the home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory");

    // Load the config
    let working_dir = home_dir.join(".cbuilder");
    let props_file_path = working_dir.join("properties.yml");
    let mut config =
        Config::read_from_file(&props_file_path).ok_or(StartError::CouldNotFindConfig)?;

    let ec2_client = EC2Client::new(config.get_base_profile());

    let cfn_client = CfnClient::new(config.get_base_profile());

    // Get the instance ID
    let instance_id = cfn_client
        .get_instance_id()
        .await
        .ok_or(StartError::InstanceNotFound)?;

    let started_instance = ec2_client.start_instance(instance_id.clone()).await;

    if !started_instance {
        return Err(StartError::FailedToStart);
    }

    // TODO: Fix this!
    sleep(Duration::from_secs(60));

    // Find the new IP of the instance
    let instance_ip = ec2_client
        .get_instance_ip(instance_id)
        .await
        .ok_or(StartError::DescribeInstanceFailed)?;

    config.set_instance_ip(instance_ip);

    config
        .write_to_file(&props_file_path)
        .map_err(|_| StartError::FailedSaveConfig)?;

    Ok(())
}
