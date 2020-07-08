use rusoto_cloudformation::{
    CloudFormation, CloudFormationClient, DeleteStackInput, DescribeStackResourceInput,
};
use rusoto_core::{credential::ProfileProvider, HttpClient, Region};

pub struct CfnClient {
    client: CloudFormationClient,
}

impl CfnClient {
    pub fn new(profile_name: String) -> CfnClient {
        let home_dir = dirs::home_dir().expect("Could not find home directory");

        let profile_provider =
            ProfileProvider::with_configuration(home_dir.join(".aws/credentials"), profile_name);

        // Create cloudformation client
        let client = CloudFormationClient::new_with(
            HttpClient::new().expect("Failed to create request dispatcher"),
            profile_provider.clone(),
            Region::UsEast1,
        );

        CfnClient { client }
    }

    pub async fn get_instance_id(&self) -> Option<String> {
        // Describe the stack
        let result = self
            .client
            .describe_stack_resource(DescribeStackResourceInput {
                stack_name: "container-builder".to_owned(),
                logical_resource_id: "Instance".to_owned(),
            })
            .await
            .ok()?;

        // Find the ID of the instance
        let result = result
            .stack_resource_detail
            .map(|t| t.physical_resource_id)
            .flatten()?;

        Some(result)
    }

    pub async fn delete_stack(&self, stack_name: String) -> bool {
        let result = self
            .client
            .delete_stack(DeleteStackInput {
                stack_name: "container-builder".to_owned(),
                client_request_token: None,
                retain_resources: None,
                role_arn: None,
            })
            .await;

        result.is_ok()
    }
}
