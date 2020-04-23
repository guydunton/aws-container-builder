use rusoto_cloudformation::{CloudFormation, DescribeStacksInput};

fn print_err<E: std::fmt::Display>(message: &'static str) -> impl Fn(E) -> () {
    let result = move |err| {
        println!("{} {}", message, err);
    };
    return result;
}

pub async fn get_stack_ip_address<Client: CloudFormation>(client: &Client) -> Option<String> {
    // Retrieve the instance IP
    let stack_description = client
        .describe_stacks(DescribeStacksInput {
            next_token: None,
            stack_name: Some("container-builder".to_owned()),
        })
        .await
        .map_err(print_err("Failed describe stack with err"))
        .ok()?;

    // Get the stacks from the result
    let stacks = stack_description.stacks?;

    // Get the container-builder stack
    let my_stack = stacks
        .iter()
        .find(|stack| stack.stack_name == "container-builder".to_owned())
        .or_else(|| {
            println!("Failed to find stack container-builder");
            None
        })?;

    // Get the outputs from that stack
    let outputs = my_stack.outputs.clone()?;

    // Get the IP address
    let ip_address = outputs
        .iter()
        .find(|output| output.output_key == Some("InstanceIP".to_owned()))
        .or_else(|| {
            println!("Failed to get output InstansteIP");
            None
        })?
        .output_value
        .clone();

    ip_address
}
