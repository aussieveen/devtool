use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct TokenResponse {
    pub access_token: String,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct ErrorResponse {
    pub error: i16,
    pub error_description: String,
}

#[derive(Deserialize, Debug, PartialEq)]
#[serde(untagged)]
pub enum AuthZeroResponse {
    TokenResponse(TokenResponse),
    ErrorResponse(ErrorResponse),
}

#[cfg(test)]
mod tests {
    use crate::client::auth_zero::models::TokenResponse;

    #[test]
    fn deserialize_token_response() {
        let json = r#"{
            "access_token": "token"
        }"#;
        let token_response: TokenResponse = serde_json::from_str(json).unwrap();
        assert_eq!(token_response.access_token, "token");
    }
}
