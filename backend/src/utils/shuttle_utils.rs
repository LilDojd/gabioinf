/// Retrieves secret values from the Shuttle secret store.
///
/// This function attempts to fetch three specific secrets: DOMAIN_URL, CLIENT_ID, and CLIENT_SECRET.
/// If any of these secrets are not found in the store, it defaults to the string "None".
///
/// # Arguments
///
/// * `secrets` - A `shuttle_runtime::SecretStore` instance containing the application's secrets.
///
/// # Returns
///
/// A tuple containing three `String`s in the following order:
/// 1. DOMAIN_URL
/// 2. CLIENT_ID
/// 3. CLIENT_SECRET
pub fn grab_secrets(secrets: shuttle_runtime::SecretStore) -> (String, String, String) {
    let domain = secrets
        .get("DOMAIN_URL")
        .unwrap_or_else(|| "None".to_string());

    let client_id = secrets
        .get("CLIENT_ID")
        .unwrap_or_else(|| "None".to_string());

    let client_secret = secrets
        .get("CLIENT_SECRET")
        .unwrap_or_else(|| "None".to_string());

    (domain, client_id, client_secret)
}
