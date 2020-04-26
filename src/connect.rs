use super::Config;

pub fn connect() -> String {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let working_dir = home_dir.join(".cbuilder");

    // Get the path of the ssh key
    let maybe_ssh_key = working_dir
        .join("ContainerBuilderKey.pem")
        .canonicalize()
        .ok()
        .map(|path| path.to_str().map(|s| s.to_owned()))
        .flatten();

    let ssh_key_path = match maybe_ssh_key {
        Some(path) => path,
        None => {
            return create_error_msg("Could not find SSH Key");
        }
    };

    // Load the config
    let maybe_config = Config::read_from_file(&working_dir.join("properties.yml"));

    // Get the IP access
    return maybe_config
        .map(|config| config.get_instance_ip())
        .map(|ip| format!("ssh -i {} ec2-user@{}", ssh_key_path, ip))
        .unwrap_or(create_error_msg("Failed to find IP of instance"));
}

fn create_error_msg(msg: &str) -> String {
    format!("echo '{}'", msg)
}
