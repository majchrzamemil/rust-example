use hmac::{Hmac, NewMac};
use jwt::Error;
use jwt::{SignWithKey, VerifyWithKey};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use super::error::ServiceError;

#[derive(Serialize, Deserialize)]
struct Claims {
    id: String,
}

//CANNOT BE LIKE THIS, MOVE THIS TO ENV
const SECRET: &'static str = "blaze_it";

pub fn generate_token(uuid: &String) -> Result<String, Error> {
    let key: Hmac<Sha256> = Hmac::new_varkey(SECRET.as_bytes()).unwrap();
    let claims = Claims {
        id: uuid.to_string(),
    };
    let token_str = claims.sign_with_key(&key).unwrap();
    Ok(token_str)
}

pub fn verify_token(token: &String) -> Result<String, ServiceError> {
    let key: Hmac<Sha256> = Hmac::new_varkey(SECRET.as_bytes()).unwrap();
    let claims: Result<Claims,Error> = token.verify_with_key(&key);
    match claims {
        Ok(claims) => Ok(claims.id),
        Err(_) => Err(ServiceError::AuthenticationError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn token_generation_should_succeed() {
        let some_uuid: String = "fat_og_kush".to_string();
        let token = generate_token(&some_uuid);

        let real_token: String = "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6ImZhdF9vZ19rdXNoIn0.RLSogAOVkvjr8Eo_TYeNMENty9ZCgBZx28_OhvRFIsQ"
            .to_string();
        assert_eq!(token.unwrap(), real_token);
    }

    #[test]
    fn token_validation_should_succeed() {
        let real_token: String = "eyJhbGciOiJIUzI1NiJ9.eyJpZCI6ImZhdF9vZ19rdXNoIn0.RLSogAOVkvjr8Eo_TYeNMENty9ZCgBZx28_OhvRFIsQ"
            .to_string();
        let some_uuid: String = "fat_og_kush".to_string();
        let token = verify_token(&real_token);

        assert_eq!(token.unwrap(), some_uuid);
    }

    #[test]
    fn token_validation_should_fail() {
        let real_token: String =
            "DUPA.BAD_TOKEN_hdF9vZ19rdXNoIn0.RLSogAOVkvjr8Eo_TYeNMENty9ZCgBZx28_OhvRFIsQ"
                .to_string();
        let token = verify_token(&real_token);
        match token {
            Ok(_) => assert!(false),
            Err(_) => assert!(true),
        }
    }
}
