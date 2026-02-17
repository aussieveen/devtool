use serde::Deserialize;

#[derive(Deserialize, Debug, PartialEq)]
pub struct TokenResponse {
    pub access_token: String,
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
