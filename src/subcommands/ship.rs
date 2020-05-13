use crate::{Config, DockerIgnore, SSHClient};
use flate2::write::GzEncoder;
use flate2::Compression;
use std::fs::File;
use std::fs::{read_dir, DirEntry};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum ShipError {
    NoDockerIgnore,
    RegistryUriMalformed,
    ArchiveCreationFailed(String),
    ScriptCreateFailed,
    ConfigFileNotOpened,
    ScriptExitCodeError(String),
    SendFileError(String),
}

pub fn ship(
    path: String,
    registry_uri: String,
    additional_args: Option<Vec<String>>,
    tag: String,
) -> Result<(), ShipError> {
    let target_dir = Path::new(&path);
    // Find all files in directory
    let all_files = get_all_files(target_dir);

    // Get the account from the registry_uri
    let account = registry_uri
        .split(".")
        .nth(0)
        .ok_or(ShipError::RegistryUriMalformed)
        .map(|acc| acc.to_owned())?;

    // Remove the files from .dockerignore
    let ignore = DockerIgnore::new(target_dir.join(".dockerignore"))
        .map_err(|_| ShipError::NoDockerIgnore)?;
    let filtered_files = ignore.filter_files(&all_files);

    tar_files(&filtered_files, &target_dir)
        .map_err(|err| ShipError::ArchiveCreationFailed(err.to_string()))?;

    // Write a script
    create_script(account, registry_uri, additional_args, tag)?;

    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    let config = Config::read_from_file(&working_dir.join("properties.yml"))
        .ok_or(ShipError::ConfigFileNotOpened)?;
    let ssh_client = SSHClient::new(
        config.get_instance_ip(),
        working_dir.join("ContainerBuilderKey.pem"),
    );

    // Ship archive
    ssh_client
        .send_file(Path::new("archive.tar.gz"), "archive.tar.gz".to_owned())
        .map_err(|err| ShipError::SendFileError(format!("{:#?}", err)))?;

    // Ship script
    ssh_client
        .send_file(Path::new("script.sh"), "script.sh".to_owned())
        .map_err(|err| ShipError::SendFileError(format!("{:#?}", err)))?;

    // Run script
    let was_success = ssh_client
        .run_command("./script.sh > log".to_owned())
        .map_err(|err| ShipError::ScriptExitCodeError(format!("{:#?}", err)))?;

    if was_success {
        Ok(())
    } else {
        Err(ShipError::ScriptExitCodeError(
            "Script exited with non zero error code".to_owned(),
        ))
    }
}

fn get_all_files(path: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = Vec::new();
    for maybe_dir in read_dir(path).unwrap() {
        let dir: DirEntry = maybe_dir.unwrap();
        if dir.file_type().unwrap().is_dir() {
            files.append(&mut get_all_files(&dir.path()));
        } else {
            files.push(dir.path());
        }
    }
    files
}

fn tar_files(all_files: &Vec<PathBuf>, target_dir: &Path) -> std::io::Result<()> {
    // Zip up the path
    let tar_file = File::create("archive.tar.gz")?;
    let enc = GzEncoder::new(&tar_file, Compression::default());
    let mut tar = tar::Builder::new(enc);

    if let Some(err) = all_files
        .iter()
        //.map(|f| tar.append_path(f))
        .map(|f: &PathBuf| {
            // Remove the target_dir from the front of the file for the path
            let mut file = File::open(&f)?;
            return tar.append_file(f.strip_prefix(&target_dir).unwrap(), &mut file);
        })
        .find(|r| r.is_err())
    {
        return err;
    }
    Ok(())
}

fn create_script(
    target_account: String,
    registry_uri: String,
    additional_args: Option<Vec<String>>,
    tag: String,
) -> Result<(), ShipError> {
    // Set additional args
    let build_args = match additional_args {
        Some(args) => args.join(" "),
        None => "".to_owned(),
    };

    // Create the script itself

    let mut script = Vec::new();
    script.push("#!/bin/bash -eux".to_owned());
    script.push("mkdir archive".to_owned());
    script.push("tar -xzvf archive.tar.gz -C archive".to_owned());
    script.push("cd archive".to_owned());
    script.push(format!("aws configure set profile.target_profile.role_arn arn:aws:iam::{}:role/ContainerBuilderPushRole", target_account));
    script.push(
        "aws configure set profile.target_profile.credential_source Ec2InstanceMetadata".to_owned(),
    );
    script.push(format!("aws ecr get-login-password --profile target_profile --region us-east-1 | docker login --username AWS --password-stdin {}", registry_uri));
    script.push(format!("docker build -t {} {} .", registry_uri, build_args));
    script.push(format!("docker push {}:{}", registry_uri, tag));
    script.push("cd ..".to_owned());
    script.push("rm -rf archive".to_owned());

    let script_data = script.join("\n");

    // Write into file
    let mut script_file: File =
        File::create("script.sh").map_err(|_| ShipError::ScriptCreateFailed)?;
    script_file
        .write(script_data.as_bytes())
        .map_err(|_| ShipError::ScriptCreateFailed)?;

    // Set file execute permissions
    if cfg!(unix) {
        // Set readonly permissions on the file if on unix
        let metadata = script_file
            .metadata()
            .map_err(|_| ShipError::ScriptCreateFailed)?;

        let mut permissions = metadata.permissions();
        permissions.set_mode(0o777);
        script_file
            .set_permissions(permissions)
            .map_err(|_| ShipError::ScriptCreateFailed)?;
    }

    Ok(())
}
