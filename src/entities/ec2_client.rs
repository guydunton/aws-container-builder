use rusoto_core::{credential::ProfileProvider, HttpClient, Region};
use rusoto_ec2::{
    CreateKeyPairRequest, DeleteKeyPairRequest, DescribeImagesRequest, DescribeInstancesRequest,
    Ec2, Ec2Client, Filter, Image, Instance, StartInstancesRequest, StopInstancesRequest,
};

pub struct EC2Client {
    client: Ec2Client,
}

impl EC2Client {
    pub fn new(profile: String) -> EC2Client {
        let home_dir = dirs::home_dir().expect("Could not find home directory");

        // Create EC2 client
        let profile_provider =
            ProfileProvider::with_configuration(home_dir.join(".aws/credentials"), profile);

        let client = Ec2Client::new_with(
            HttpClient::new().expect("Failed to create request dispatcher"),
            profile_provider.clone(),
            Region::UsEast1,
        );

        EC2Client { client }
    }

    pub async fn start_instance(&self, instance_id: String) -> bool {
        let result = self
            .client
            .start_instances(StartInstancesRequest {
                dry_run: Some(false),
                additional_info: None,
                instance_ids: vec![instance_id],
            })
            .await;

        result.is_ok()
    }

    pub async fn stop_instance(&self, instance_id: String) -> bool {
        let result = self
            .client
            .stop_instances(StopInstancesRequest {
                dry_run: Some(false),
                instance_ids: vec![instance_id],
                ..StopInstancesRequest::default()
            })
            .await;

        result.is_ok()
    }

    pub async fn get_instance_ip(&self, instance_id: String) -> Option<String> {
        let result = self
            .client
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

    pub async fn delete_key_pair(&self, key_pair_name: String) -> bool {
        let result = self
            .client
            .delete_key_pair(DeleteKeyPairRequest {
                key_name: Some(key_pair_name),
                dry_run: Some(false),
                key_pair_id: None,
            })
            .await;

        result.is_ok()
    }

    pub async fn create_ssh_key(&self) -> Option<String> {
        let result = self
            .client
            .create_key_pair(CreateKeyPairRequest {
                dry_run: Some(false),
                key_name: "ContainerBuilderKey".to_owned(),
                tag_specifications: None,
            })
            .await;

        result.ok().map(|key| key.key_material).flatten()
    }

    pub async fn get_amazon_linux_2_ami(&self) -> Option<Vec<Image>> {
        // Find the correct AWS Amazon Linux 2 AMI
        let images_request = self
            .client
            .describe_images(DescribeImagesRequest {
                dry_run: Some(false),
                filters: Some(vec![
                    create_filter("name", "amzn2-ami-hvm-2.0.????????.?-x86_64-gp2"),
                    create_filter("state", "available"),
                ]),
                owners: Some(vec![String::from("amazon")]),
                ..DescribeImagesRequest::default()
            })
            .await;

        // Pull out the images and check that they exist
        let images = images_request.ok()?.images;

        images
    }
}

fn create_filter(name: &str, value: &str) -> Filter {
    Filter {
        name: Some(name.to_owned()),
        values: Some(vec![value.to_owned()]),
    }
}
