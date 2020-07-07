use crate::{get_instance_id, Config};
use rusoto_cloudformation::CloudFormationClient;
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_ec2::{DescribeInstancesRequest, Ec2, Ec2Client, Instance, StartInstancesRequest};
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

    // Create EC2 client
    let profile_provider = ProfileProvider::with_configuration(
        home_dir.join(".aws/credentials"),
        config.get_base_profile(),
    );

    // delete the ssh key
    let ec2_client = Ec2Client::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    // Create cloudformation client
    let cfn_client = CloudFormationClient::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    // Get the instance ID
    let instance_id = get_instance_id(&cfn_client)
        .await
        .ok_or(StartError::InstanceNotFound)?;

    // Start the instance
    ec2_client
        .start_instances(StartInstancesRequest {
            dry_run: Some(false),
            additional_info: None,
            instance_ids: vec![instance_id.clone()],
        })
        .await
        .map_err(|_| StartError::FailedToStart)?;

    // TODO: Fix this!
    sleep(Duration::from_secs(60));

    // Find the new IP of the instance
    let instance_ip = get_instance_ip(&ec2_client, instance_id)
        .await
        .ok_or(StartError::DescribeInstanceFailed)?;

    config.set_instance_ip(instance_ip);

    config
        .write_to_file(&props_file_path)
        .map_err(|_| StartError::FailedSaveConfig)?;

    Ok(())
}

async fn get_instance_ip<Client: Ec2>(client: &Client, instance_id: String) -> Option<String> {
    let result = client
        .describe_instances(DescribeInstancesRequest {
            dry_run: Some(false),
            instance_ids: Some(vec![instance_id]),
            ..DescribeInstancesRequest::default()
        })
        .await
        .ok()?;

    let instances: Vec<Instance> = result
        .reservations
        .unwrap()
        .into_iter()
        .flat_map(|res| res.instances)
        .flatten()
        .collect();

    instances
        .first()
        .map(|instance| instance.public_ip_address.clone())
        .flatten()
}
