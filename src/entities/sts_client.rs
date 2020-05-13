use rusoto_core::credential::ProfileProvider;
use rusoto_core::{HttpClient, Region};
use rusoto_sts::{GetCallerIdentityRequest, Sts, StsClient};

pub async fn get_current_account_no(profile: String) -> Result<String, String> {
    let home_dir = dirs::home_dir().expect("Could not find home directory");
    let profile_provider =
        ProfileProvider::with_configuration(home_dir.join(".aws/credentials"), profile.clone());

    let client = StsClient::new_with(
        HttpClient::new().expect("Failed to create request dispatcher"),
        profile_provider.clone(),
        Region::UsEast1,
    );

    let response = client
        .get_caller_identity(GetCallerIdentityRequest {})
        .await
        .map_err(|err| err.to_string())?;

    let account_id = response
        .account
        .ok_or("account not found on response".to_owned())?;

    Ok(account_id)
}
