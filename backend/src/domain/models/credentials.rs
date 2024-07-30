use oauth2::CsrfToken;
use serde::Deserialize;

/// Represents the credentials received during the OAuth2 authorization code flow.
///
/// This struct is used to capture and validate the response from an OAuth2 authorization server.
/// It includes the authorization code and CSRF tokens for security verification.
#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    /// The authorization code received from the OAuth2 server.
    ///
    /// This code is typically exchanged for an access token in the next step of the OAuth2 flow.
    pub code: String,

    /// The original CSRF token sent in the initial authorization request.
    ///
    /// This token is used to prevent CSRF attacks by ensuring the response
    /// corresponds to a request initiated by this application.
    pub old_state: CsrfToken,

    /// The new CSRF token received in the authorization response.
    ///
    /// This should match the `old_state` to verify the integrity of the OAuth2 flow.
    pub new_state: CsrfToken,
}
