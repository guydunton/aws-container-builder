use rusoto_cloudformation::{
    CloudFormation, CreateStackError, CreateStackInput, DescribeStackEventsInput,
};
use std::thread::sleep;
use std::time::{Duration, Instant};

use super::Tag;
use rusoto_core::RusotoError;

#[derive(Debug)]
pub enum DeployError {
    CreateStackFailed,
    DescribeStackFailed,
    TimedOut,
}

impl std::fmt::Display for DeployError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Must be :? or this will be recursive and cause a stack overflow
        write!(f, "{:?}", self)
    }
}

#[derive(Clone)]
pub struct Parameter {
    key: String,
    value: String,
}

impl Parameter {
    pub fn new(key: String, value: String) -> Parameter {
        Parameter { key, value }
    }
}

pub async fn deploy_stack<Client: CloudFormation>(
    client: &Client,
    stack_name: String,
    template: String,
    parameters: &Vec<Parameter>,
    tags: &Vec<Tag>,
    timeout: u64,
) -> Result<(), DeployError> {
    // Start the stack creation
    let create_result = client
        .create_stack(CreateStackInput {
            capabilities: Some(vec!["CAPABILITY_NAMED_IAM".to_owned()]),
            stack_name: stack_name.clone(),
            tags: Some(tags.iter().map(|tag| convert_tag(tag.clone())).collect()),
            parameters: Some(
                parameters
                    .iter()
                    .map(|param| convert_parameter(param.clone()))
                    .collect(),
            ),
            template_body: Some(template),
            ..CreateStackInput::default()
        })
        .await;

    // If the stack already exists then go on to wait until it's available
    match create_result {
        Ok(_) => {}
        Err(RusotoError::Service(CreateStackError::AlreadyExists(_))) => {}
        _ => {
            // Fail
            return Err(DeployError::CreateStackFailed);
        }
    }

    let start_time = Instant::now();

    // Loop until creation has completed or timed out
    loop {
        let created_stack = has_stack_completed(client, &stack_name).await?;

        if created_stack {
            break;
        }

        // Check for timeout
        if start_time.elapsed() > Duration::from_secs(timeout) {
            return Err(DeployError::TimedOut);
        }

        sleep(Duration::from_secs(5));
    }

    Ok(())
}

async fn has_stack_completed<Client: CloudFormation>(
    client: &Client,
    stack_name: &String,
) -> Result<bool, DeployError> {
    // This function assumes that the stack has just been created

    let describe_stack_events_output = client
        .describe_stack_events(DescribeStackEventsInput {
            stack_name: Some(stack_name.clone()),
            next_token: None,
        })
        .await;

    let stack_events = describe_stack_events_output
        .map_err(|_| DeployError::DescribeStackFailed)?
        .stack_events
        .ok_or(DeployError::DescribeStackFailed)?;

    // resource_status 'CREATE_COMPLETE' | 'UPDATE_COMPLETE' & logical_resource_id == 'container-builder'
    let created_stack = stack_events.iter().any(|event| {
        event.logical_resource_id == Some(stack_name.clone())
            && (event.resource_status == Some("CREATE_COMPLETE".to_owned())
                || event.resource_status == Some("UPDATE_COMPLETE".to_owned()))
    });

    Ok(created_stack)
}

fn convert_tag(tag: Tag) -> rusoto_cloudformation::Tag {
    rusoto_cloudformation::Tag {
        key: tag.key,
        value: tag.value,
    }
}

fn convert_parameter(parameter: Parameter) -> rusoto_cloudformation::Parameter {
    rusoto_cloudformation::Parameter {
        parameter_key: Some(parameter.key),
        parameter_value: Some(parameter.value),
        ..rusoto_cloudformation::Parameter::default()
    }
}
