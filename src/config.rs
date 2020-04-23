use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    instance_ip: String,
    base_profile: String,
}

pub enum ConfigWriteError {
    ParsingFailed(serde_yaml::Error),
    FileOperationFailed(std::io::Error),
}

impl Config {
    pub fn new(instance_ip: String, base_profile: String) -> Config {
        Config {
            instance_ip,
            base_profile,
        }
    }

    pub fn get_instance_ip(&self) -> String {
        self.instance_ip.clone()
    }

    pub fn get_base_profile(&self) -> String {
        self.base_profile.clone()
    }

    pub fn write_to_file(&self, path: &Path) -> Result<(), ConfigWriteError> {
        let data =
            serde_yaml::to_string(self).map_err(|err| ConfigWriteError::ParsingFailed(err))?;
        let mut file: File =
            File::create(path).map_err(|err| ConfigWriteError::FileOperationFailed(err))?;

        file.write_all(data.as_bytes())
            .map_err(|err| ConfigWriteError::FileOperationFailed(err))?;

        Ok(())
    }

    pub fn read_from_file(path: &Path) -> Option<Config> {
        let data = std::fs::read_to_string(path).ok()?;

        let config = serde_yaml::from_str(&data).ok()?;

        Some(config)
    }
}
