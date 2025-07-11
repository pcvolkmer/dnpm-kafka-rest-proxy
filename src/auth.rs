use base64::prelude::BASE64_STANDARD;
use base64::Engine;

#[allow(clippy::module_name_repetitions)]
pub fn check_basic_auth(auth_header: &str, expected_token: &str) -> bool {
    let split = auth_header.split(' ').collect::<Vec<_>>();
    if split.len() == 2 && split.first().map(|first| first.to_lowercase()) == Some("basic".into()) {
        if let Ok(auth) = BASE64_STANDARD.decode(split.last().unwrap_or(&"")) {
            if let Ok(auth) = String::from_utf8(auth) {
                let split = auth.split(':').collect::<Vec<_>>();
                if split.len() == 2 && split.first() == Some(&"token") {
                    match split.last() {
                        None => {}
                        Some(&token) => {
                            if let Ok(true) = bcrypt::verify(token, expected_token) {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::auth::check_basic_auth;

    // plain text value 'very-secret'
    const EXPECTED_TOKEN: &str = "$2y$05$LIIFF4Rbi3iRVA4UIqxzPeTJ0NOn/cV2hDnSKFftAMzbEZRa42xSG";

    #[test]
    fn should_reject_non_basic_header_content() {
        assert!(!check_basic_auth("token 123456789", EXPECTED_TOKEN));
    }

    #[test]
    fn should_reject_invalid_basic_auth() {
        assert!(!check_basic_auth("Basic 123456789", EXPECTED_TOKEN));
    }

    #[test]
    fn should_reject_basic_auth_without_token_username() {
        assert!(!check_basic_auth(
            "Basic dXNlcjoxMjM0NTY3ODk=",
            EXPECTED_TOKEN
        ));
    }

    #[test]
    fn should_reject_basic_auth_without_wrong_token() {
        assert!(!check_basic_auth(
            "Basic dG9rZW46MTIzNDU2Nzg5",
            EXPECTED_TOKEN
        ));
    }

    #[test]
    fn should_accept_basic_auth_without_correct_token() {
        assert!(check_basic_auth(
            "Basic dG9rZW46dmVyeS1zZWNyZXQ=",
            EXPECTED_TOKEN
        ));
    }
}
