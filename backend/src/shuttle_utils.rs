#[doc(hidden)]
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
