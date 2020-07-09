use chrono::prelude::*;
use core::str::FromStr;
use rusoto_ec2::Image;
use std::fs::{create_dir, read_to_string, File};
use std::io::prelude::*;
use std::ops::Fn;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use crate::{get_current_account_no, CfnClient, Config, EC2Client, SimpleParameter, Tag};

#[derive(Debug)]
pub enum BootstrapErrors {
    FailedCreateWorkingDir(String),
    FailedCreatingSSHKey(String),
    FailedFindAmi,
    FailedStackCreation(String),
    FailedDescribeStack,
    FailedWriteConfig,
    FailedSetKeyPermissions,
    FailedGetCurrentAccountId(String),
}

pub async fn run_bootstrap(profile: String, tags: Vec<Tag>) -> Result<(), BootstrapErrors> {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    ensure_working_dir_exists(working_dir.clone(), create_dir)
        .map_err(|err| BootstrapErrors::FailedCreateWorkingDir(err))?;

    // Create SSH Key into directory
    let my_ec2 = EC2Client::new(profile.clone());
    let key = my_ec2
        .create_ssh_key()
        .await
        .ok_or(BootstrapErrors::FailedCreatingSSHKey(
            "Failed create ec2 ssh key".to_owned(),
        ))?;
    create_ssh_key(key, working_dir.join("ContainerBuilderKey.pem")).await?;

    // Find the correct AWS Amazon Linux 2 AMI
    let images = my_ec2
        .get_amazon_linux_2_ami()
        .await
        .ok_or(BootstrapErrors::FailedFindAmi)?;
    let linux_ami = get_amazon_linux_2_ami(images).await?;

    // Deploy cloudformation stack
    let cfn_client = CfnClient::new(profile.clone());

    // Load the cloudformation template file to a string
    let cfn_template = read_to_string("resources/instance-cfn.yml")
        .map_err(|err| BootstrapErrors::FailedStackCreation(err.to_string()))?;

    // Get the current account id
    let account_id = get_current_account_no(profile.clone())
        .await
        .map_err(|err| BootstrapErrors::FailedGetCurrentAccountId(err))?;
    let role = format!("arn:aws:iam::{}:role/ContainerBuilderPushRole", account_id);

    // Deploy the cloudformation template
    cfn_client
        .deploy_stack(
            "container-builder".to_owned(),
            cfn_template,
            &vec![
                SimpleParameter::new("AmiId".to_owned(), linux_ami),
                SimpleParameter::new("SSHKeyName".to_owned(), "ContainerBuilderKey".to_owned()),
                SimpleParameter::new("AccountRoles".to_owned(), role),
            ],
            &tags,
            7 * 60,
        )
        .await
        .map_err(|err| BootstrapErrors::FailedStackCreation(err.to_string()))?;

    let instance_id = cfn_client
        .get_instance_id()
        .await
        .ok_or(BootstrapErrors::FailedDescribeStack)?;

    let ec2_client = EC2Client::new(profile.clone());
    let instance_ip = ec2_client
        .get_instance_ip(instance_id)
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
            err.to_string()
        )
    })
}

pub async fn get_amazon_linux_2_ami(mut images: Vec<Image>) -> Result<String, BootstrapErrors> {
    //    .ok_or(BootstrapErrors::FailedFindAmi)?;

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

pub async fn create_ssh_key(key: String, path: PathBuf) -> Result<(), BootstrapErrors> {
    // If the key exists then don't recreate it
    if path.exists() {
        return Ok(());
    }

    // Create the ssh key file
    let mut file =
        File::create(path).map_err(|err| BootstrapErrors::FailedCreatingSSHKey(err.to_string()))?;

    // Store the key contents
    file.write_all(key.as_bytes()).map_err(|_| {
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
