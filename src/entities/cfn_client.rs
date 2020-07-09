use crate::Tag;
use rusoto_cloudformation::{
    CloudFormation, CloudFormationClient, CreateStackError, CreateStackInput, DeleteStackInput,
    DescribeStackEventsInput, DescribeStackResourceInput, Parameter, UpdateStackInput,
};
use rusoto_core::{credential::ProfileProvider, HttpClient, Region, RusotoError};
use std::{
    thread::sleep,
    time::{Duration, Instant},
};

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
pub struct SimpleParameter {
    key: String,
    value: String,
}

impl SimpleParameter {
    pub fn new(key: String, value: String) -> SimpleParameter {
        SimpleParameter { key, value }
    }
}

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
                stack_name,
                client_request_token: None,
                retain_resources: None,
                role_arn: None,
            })
            .await;

        result.is_ok()
    }

    pub async fn update_stack(&self, accounts: String) -> bool {
        let deploy_result = self
            .client
            .update_stack(UpdateStackInput {
                capabilities: Some(vec!["CAPABILITY_NAMED_IAM".to_owned()]),
                stack_name: "container-builder".to_owned(),
                parameters: Some(vec![
                    Parameter {
                        parameter_key: Some("AmiId".to_owned()),
                        use_previous_value: Some(true),
                        ..Parameter::default()
                    },
                    Parameter {
                        parameter_key: Some("SSHKeyName".to_owned()),
                        use_previous_value: Some(true),
                        ..Parameter::default()
                    },
                    Parameter {
                        parameter_key: Some("AccountRoles".to_owned()),
                        parameter_value: Some(accounts),
                        ..Parameter::default()
                    },
                ]),
                use_previous_template: Some(true),
                ..UpdateStackInput::default()
            })
            .await;

        deploy_result.is_ok()
    }

    pub async fn deploy_stack(
        &self,
        stack_name: String,
        template: String,
        parameters: &Vec<SimpleParameter>,
        tags: &Vec<Tag>,
        timeout: u64,
    ) -> Result<(), DeployError> {
        // Start the stack creation
        let create_result = self
            .client
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
            let created_stack = has_stack_completed(&self.client, &stack_name).await?;

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

fn convert_parameter(parameter: SimpleParameter) -> rusoto_cloudformation::Parameter {
    rusoto_cloudformation::Parameter {
        parameter_key: Some(parameter.key),
        parameter_value: Some(parameter.value),
        ..rusoto_cloudformation::Parameter::default()
    }
}
