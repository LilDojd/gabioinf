#[doc(hidden)]
pub fn grab_secrets(secrets: shuttle_runtime::SecretStore) -> (String) {
    let domain = secrets
        .get("DOMAIN_URL")
        .unwrap_or_else(|| "None".to_string());

    (domain)
}
