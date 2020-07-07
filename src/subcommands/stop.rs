use crate::{get_instance_id, Config};
use rusoto_cloudformation::CloudFormationClient;
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_ec2::{Ec2, Ec2Client, StopInstancesRequest};

pub async fn run_stop() -> Result<(), String> {
    // Get the home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    // Load the config
    let config = Config::read_from_file(&working_dir.join("properties.yml"))
        .expect("Could not load config file");

    // Create EC2 client
    let profile_provider = ProfileProvider::with_configuration(
        home_dir.join(".aws/credentials"),
        config.get_base_profile(),
    );

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

    // Find the instance id
    let instance_id = get_instance_id(&cfn_client)
        .await
        .ok_or("Failed to get instance Id".to_owned())?;

    // Stop the instance
    ec2_client
        .stop_instances(StopInstancesRequest {
            dry_run: Some(false),
            instance_ids: vec![instance_id],
            ..StopInstancesRequest::default()
        })
        .await
        .map_err(|err| err.to_string())
        .map(|_| ())
}
