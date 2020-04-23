use chrono::prelude::*;
use core::str::FromStr;
use rusoto_cloudformation::CloudFormationClient;
use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_ec2::{CreateKeyPairRequest, DescribeImagesRequest, Ec2, Ec2Client, Filter};
use std::error::Error;
use std::fs::{create_dir, read_to_string, File};
use std::io::prelude::*;
use std::ops::Fn;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use super::cfn_deploy::{deploy_stack, Parameter};
use super::get_stack_ip::get_stack_ip_address;
use super::Config;
use super::Tag;

#[derive(Debug)]
pub enum BootstrapErrors {
    FailedCreateWorkingDir(String),
    FailedCreatingSSHKey(String),
    FailedFindAmi,
    FailedStackCreation(String),
    FailedDescribeStack,
    FailedWriteConfig,
    FailedSetKeyPermissions,
}

pub async fn run_bootstrap(profile: String, tags: Vec<Tag>) -> Result<(), BootstrapErrors> {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    ensure_working_dir_exists(working_dir.clone(), create_dir)
        .map_err(|err| BootstrapErrors::FailedCreateWorkingDir(err))?;

    let profile_provider =
        ProfileProvider::with_configuration(home_dir.join(".aws/credentials"), profile.clone());

    let ec2_client = Ec2Client::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    // Create SSH Key into directory
    create_ssh_key(&ec2_client, working_dir.join("ContainerBuilderKey.pem")).await?;

    // Find the correct AWS Amazon Linux 2 AMI
    let linux_ami = get_amazon_linux_2_ami(&ec2_client).await?;

    // Deploy cloudformation stack
    let cfn_client = CloudFormationClient::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    // Load the cloudformation template file to a string
    let cfn_template = read_to_string("resources/instance-cfn.yml")
        .map_err(|err| BootstrapErrors::FailedStackCreation(err.to_string()))?;

    // Deploy the cloudformation template
    deploy_stack(
        &cfn_client,
        "container-builder".to_owned(),
        cfn_template,
        &vec![
            Parameter::new("AmiId".to_owned(), linux_ami),
            Parameter::new("SSHKeyName".to_owned(), "ContainerBuilderKey".to_owned()),
        ],
        &tags,
        7 * 60,
    )
    .await
    .map_err(|err| BootstrapErrors::FailedStackCreation(err.to_string()))?;

    // Get the IP address of the instance
    let instance_ip = get_stack_ip_address(&cfn_client)
        .await
        .ok_or(BootstrapErrors::FailedDescribeStack)?;

    let config = Config::new(instance_ip, profile.clone());
    config
        .write_to_file(&working_dir.join("properties.yml"))
        .map_err(|_| BootstrapErrors::FailedWriteConfig)?;

    return Ok(());
}

trait HasExists {
    fn exists(&self) -> bool;
}

pub fn ensure_working_dir_exists<F>(working_dir: PathBuf, create_dir_fn: F) -> Result<(), String>
where
    F: Fn(PathBuf) -> std::io::Result<()>,
{
    // Create working directory if it doesn't exist
    if working_dir.exists() {
        return Ok(());
    }

    // Create directory. If fails now then we know there is some
    // other kind of issue and we can return the error
    create_dir_fn(working_dir).map_err(|err| {
        format!(
            "Failed to create working directory with error: {}",
            err.description()
        )
    })
}

fn create_filter(name: &str, value: &str) -> Filter {
    Filter {
        name: Some(name.to_owned()),
        values: Some(vec![value.to_owned()]),
    }
}

pub async fn get_amazon_linux_2_ami<Client: Ec2>(
    client: &Client,
) -> Result<String, BootstrapErrors> {
    // Find the correct AWS Amazon Linux 2 AMI
    let images_request = client
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
    let mut images = images_request
        .map_err(|_| BootstrapErrors::FailedFindAmi)?
        .images
        .ok_or(BootstrapErrors::FailedFindAmi)?;

    // Check creation dates & image_ids are set
    if images
        .iter()
        .any(|image| image.creation_date.is_none() || image.image_id.is_none())
    {
        return Err(BootstrapErrors::FailedFindAmi);
    }

    // Sort into newest AMI first
    images.sort_unstable_by(|a, b| {
        // There are a lot of unwraps here which have been partially checked above.
        // The solution is to convert each image into another struct with just a creation_date
        // & image Id. This conversion is where you could handle any failures.
        let a_date = DateTime::<Local>::from_str(&a.creation_date.clone().unwrap()).unwrap();
        let b_date = DateTime::<Local>::from_str(&b.creation_date.clone().unwrap()).unwrap();

        b_date.partial_cmp(&a_date).unwrap()
    });

    // Pull the image_id from the first image
    images
        .first()
        .ok_or(BootstrapErrors::FailedFindAmi)?
        .image_id
        .clone()
        .ok_or(BootstrapErrors::FailedFindAmi)
}

pub async fn create_ssh_key<Client: Ec2>(
    client: &Client,
    path: PathBuf,
) -> Result<(), BootstrapErrors> {
    // Create SSH key within AWS
    let result = client
        .create_key_pair(CreateKeyPairRequest {
            dry_run: Some(false),
            key_name: "ContainerBuilderKey".to_owned(),
        })
        .await;

    // Check for errors in the response
    let key_pair = result
        .map_err(|err| BootstrapErrors::FailedCreatingSSHKey(err.description().to_owned()))?;

    // Get the key text
    let ssh_key = key_pair
        .key_material
        .ok_or(BootstrapErrors::FailedCreatingSSHKey(
            "Did not retrieve key_material".to_owned(),
        ))?;

    // Create the ssh key file
    let mut file = File::create(path)
        .map_err(|err| BootstrapErrors::FailedCreatingSSHKey(err.description().to_owned()))?;

    // Store the key contents
    file.write_all(ssh_key.as_bytes()).map_err(|_| {
        BootstrapErrors::FailedCreatingSSHKey("Failed to write ssh key bytes".to_owned())
    })?;

    // Set the permissions of the file to readonly
    set_file_permissions(&file);

    return Ok(());
}

fn set_file_permissions(file: &File) -> bool {
    if cfg!(unix) {
        // Set readonly permissions on the file if on unix
        let maybe_metadata = file
            .metadata()
            .map_err(|_| BootstrapErrors::FailedSetKeyPermissions);

        let metadata = match maybe_metadata {
            Ok(md) => md,
            Err(_) => {
                return false;
            }
        };

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o400);
        match file.set_permissions(permissions) {
            Ok(_) => {
                return true;
            }
            _ => {
                return false;
            }
        }
    } else {
        return true;
    }
}
