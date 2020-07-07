use rusoto_cloudformation::{CloudFormation, DescribeStackResourceInput};

pub async fn get_instance_id<Client: CloudFormation>(client: &Client) -> Option<String> {
    // Describe the stack
    let result = client
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
