use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Serialize, Deserialize, Debug)]
struct Account {
    profile: String,
    account_no: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    instance_ip: String,
    base_profile: String,
    sub_accounts: Vec<Account>,
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
            sub_accounts: vec![],
        }
    }

    pub fn get_instance_ip(&self) -> String {
        self.instance_ip.clone()
    }

    pub fn get_base_profile(&self) -> String {
        self.base_profile.clone()
    }

    pub fn get_account_numbers(&self) -> Vec<String> {
        self.sub_accounts
            .iter()
            .map(|account| account.account_no.clone())
            .collect()
    }

    pub fn add_account(&mut self, profile: String, account_no: String) {
        // Remove the current account value if exists
        let found_index = self
            .sub_accounts
            .iter()
            .position(|acc| acc.account_no == account_no);
        if let Some(index) = found_index {
            self.sub_accounts.remove(index);
        }

        // Push the
        self.sub_accounts.push(Account {
            profile,
            account_no,
        });
    }

    pub fn get_account_profile(&self, account_no: String) -> Option<String> {
        self.sub_accounts
            .iter()
            .find(|acc| acc.account_no == account_no)
            .map(|acc| acc.profile.clone())
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

    pub fn set_instance_ip(&mut self, instance_ip: String) {
        self.instance_ip = instance_ip;
    }
}

#[test]
fn adding_a_duplicate_account_updates_the_profile() {
    let mut config = Config::new("0.0.0.0".to_owned(), "base".to_owned());
    config.add_account("profile2".to_owned(), "123".to_owned());
    assert_eq!(
        config.get_account_profile("123".to_owned()),
        Some("profile2".to_owned())
    );

    config.add_account("profile3".to_owned(), "123".to_owned());
    assert_eq!(
        config.get_account_profile("123".to_owned()),
        Some("profile3".to_owned())
    );
}
