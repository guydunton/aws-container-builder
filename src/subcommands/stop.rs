use crate::{CfnClient, Config, EC2Client};

pub async fn run_stop() -> Result<(), String> {
    // Get the home directory
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    // Load the config
    let config = Config::read_from_file(&working_dir.join("properties.yml"))
        .expect("Could not load config file");

    let ec2_client = EC2Client::new(config.get_base_profile());

    let cfn_client = CfnClient::new(config.get_base_profile());

    // Find the instance id
    let instance_id = cfn_client
        .get_instance_id()
        .await
        .ok_or("Failed to get instance Id".to_owned())?;

    // Stop the instance
    let stopped_instance = ec2_client.stop_instance(instance_id).await;

    if !stopped_instance {
        return Err("Failed to stop instance".to_owned());
    } else {
        return Ok(());
    }
}
