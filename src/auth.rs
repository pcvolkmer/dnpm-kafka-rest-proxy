use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use bcrypt::HashParts;
use std::str::FromStr;

pub fn split_username_password(auth: &str) -> (String, String) {
    let split = auth.split(':').collect::<Vec<_>>();
    if split.len() == 2 {
        return (split[0].into(), split[1].into());
    }
    ("token".into(), auth.into())
}

pub fn is_valid_brypt_hash(auth: &str) -> bool {
    let (_, hash) = split_username_password(auth);
    HashParts::from_str(&hash).is_ok()
}

#[allow(clippy::module_name_repetitions)]
pub fn check_basic_auth(auth_header: &str, expected_token: &str) -> bool {
    let split = auth_header.split(' ').collect::<Vec<_>>();
    if split.len() == 2
        && split.first().map(|first| first.to_lowercase()) == Some("basic".into())
        && let Ok(auth) = BASE64_STANDARD.decode(split.last().unwrap_or(&""))
        && let Ok(auth) = String::from_utf8(auth)
    {
        let split_token = expected_token.split(':').collect::<Vec<_>>();
        let expected_username = if split_token.len() == 2 {
            split_token.first().unwrap_or(&"token")
        } else {
            &"token"
        };
        let expected_token = split_token.get(1).unwrap_or(&expected_token);

        let (username, hash) = &split_username_password(&auth);
        if username == expected_username
            && let Ok(true) = bcrypt::verify(hash, expected_token)
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use crate::auth::{check_basic_auth, is_valid_brypt_hash, split_username_password};
    use rstest::rstest;

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
    fn should_reject_basic_auth_without_valid_token() {
        assert!(!check_basic_auth(
            "Basic dG9rZW46MTIzNDU2Nzg5",
            EXPECTED_TOKEN
        ));
    }

    #[test]
    fn should_accept_basic_auth_with_valid_token() {
        assert!(check_basic_auth(
            "Basic dG9rZW46dmVyeS1zZWNyZXQ=",
            EXPECTED_TOKEN
        ));
    }

    #[test]
    fn should_accept_basic_auth_with_custom_username() {
        assert!(check_basic_auth(
            "Basic Y3VzdG9tdXNlcjp2ZXJ5LXNlY3JldA==",
            &format!("customuser:{EXPECTED_TOKEN}")
        ));
    }

    #[test]
    fn should_reject_basic_auth_without_correct_custom_username() {
        assert!(!check_basic_auth(
            "Basic Y3VzdG9tdXNlcjp2ZXJ5LXNlY3JldA==",
            &format!("otheruser:{EXPECTED_TOKEN}")
        ));
    }

    #[rstest]
    #[case("very-secret", false)]
    #[case(
        "$5$tAgL6oMasYMQidQG$PBPbP5h2/oqALm4HE7xmC.QTcGgm80s/WOStpHNRW5.",
        false
    )]
    #[case("$apr1$5p3oZQrJ$3MW2pYhmLrmJO0ELJjtnK.", false)]
    #[case(
        "b644133604bf99632137be3e9230c4056bd32eb2f404020d70adcde88353c760",
        false
    )]
    #[case("$2y$05$3oByqJRY.gB0.I6u1ng7ze/55FZvaIt9blfGvEj4zg6pZJvKC66na", true)]
    #[case(
        "test:$5$tAgL6oMasYMQidQG$PBPbP5h2/oqALm4HE7xmC.QTcGgm80s/WOStpHNRW5.",
        false
    )]
    #[case("test:$apr1$5p3oZQrJ$3MW2pYhmLrmJO0ELJjtnK.", false)]
    #[case(
        "test:b644133604bf99632137be3e9230c4056bd32eb2f404020d70adcde88353c760",
        false
    )]
    #[case(
        "test:$2y$05$3oByqJRY.gB0.I6u1ng7ze/55FZvaIt9blfGvEj4zg6pZJvKC66na",
        true
    )]
    fn should_check_bcrypt_hash_strings(#[case] input: &str, #[case] expected: bool) {
        assert_eq!(is_valid_brypt_hash(input), expected);
    }

    #[rstest]
    #[case(
        "customusername:$2y$05$3oByqJRY.gB0.I6u1ng7ze/55FZvaIt9blfGvEj4zg6pZJvKC66na",
        ("customusername".into(), "$2y$05$3oByqJRY.gB0.I6u1ng7ze/55FZvaIt9blfGvEj4zg6pZJvKC66na".into())
    )]
    // Old behavior: use "token" as default username
    #[case(
        "$2y$05$3oByqJRY.gB0.I6u1ng7ze/55FZvaIt9blfGvEj4zg6pZJvKC66na",
        ("token".into(), "$2y$05$3oByqJRY.gB0.I6u1ng7ze/55FZvaIt9blfGvEj4zg6pZJvKC66na".into())
    )]
    fn should_split_username_tokenhash(#[case] input: &str, #[case] expected: (String, String)) {
        assert_eq!(split_username_password(input), expected);
    }
}
