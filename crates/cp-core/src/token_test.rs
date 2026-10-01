use super::*;

fn jwt(payload: &str) -> String {
    let encode = |raw: &str| {
        const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let bytes = raw.as_bytes();
        let mut out = String::new();
        for chunk in bytes.chunks(3) {
            let mut buffer = 0u32;
            for (i, b) in chunk.iter().enumerate() {
                buffer |= u32::from(*b) << (16 - 8 * i);
            }
            for i in 0..=chunk.len() {
                let index = (buffer >> (18 - 6 * i)) & 63;
                out.push(TABLE[index as usize] as char);
            }
        }
        out
    };
    format!(
        "{}.{}.sig",
        encode(r#"{"alg":"HS256","typ":"JWT"}"#),
        encode(payload)
    )
}

#[test]
fn a_jwt_is_a_token_and_says_what_it_carries() {
    let token = jwt(r#"{"sub":"cp-3","env":"staging","exp":1700000000,"iss":"cp"}"#);
    assert!(looks_like(&token));
    let claims = claims_of(&token).expect("reads");
    assert_eq!(claims.subject(), Some("cp-3"));
    assert_eq!(claims.issuer(), Some("cp"));
    assert_eq!(claims.expires_at(), Some(1_700_000_000));
    assert_eq!(claims.expired_by(1_700_000_001), Some(true));
    assert_eq!(
        claims.expired_by(1_700_000_000),
        Some(true),
        "right at the exact second, it no longer holds"
    );
    assert_eq!(claims.expired_by(1_699_999_999), Some(false));
    assert_eq!(claims.expired_by(1_600_000_000), Some(false));
    assert_eq!(
        claims.payload.get("env").and_then(|v| v.as_str()),
        Some("staging"),
        "the real problem isn't seeing it: it's knowing which of the twelve is staging"
    );
}

#[test]
fn a_jwt_without_expiry_does_not_pretend_to_know() {
    let claims = claims_of(&jwt(r#"{"sub":"x"}"#)).expect("reads");
    assert_eq!(claims.expired_by(0), None);
}

#[test]
fn three_dotted_words_are_not_a_jwt() {
    assert!(claims_of("one.two.three").is_none(), "not base64 of a JSON");
    assert!(claims_of("a.b").is_none(), "parts are missing");
    assert!(
        claims_of(&format!("{}.extra", jwt("{}"))).is_none(),
        "one part too many"
    );
    assert!(!looks_like("one.two.three"));
    assert!(!looks_like("archive.tar.gz"));
}

#[test]
fn a_header_without_alg_is_just_json_in_base64() {
    let no_alg = format!("{}.firma", {
        let t = jwt("{}");
        t.replace("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9", "e30")
    });
    assert!(claims_of(&no_alg).is_none());
}

#[test]
fn the_known_prefixes_are_tokens_when_long_enough() {
    for token in [
        "sk-not-a-real-key-for-tests-0123456789",
        "sk-ant-not-a-real-key-for-tests-0000000000000000000",
        "ghp_not_a_real_token_for_tests_0000000000",
        "github_pat_not_a_real_token_for_tests_0000000000",
        "xoxb-not-a-real-slack-token-for-tests",
        "AKIAIOSFODNN7EXAMPLE",
        "AIza.not.a.real.google.key.for.tests.0000000",
        "glpat-not-a-real-token-for-tests",
    ] {
        assert!(looks_like(token), "{token}");
    }
}

#[test]
fn a_prefix_alone_or_with_spaces_is_not_a_token() {
    assert!(!looks_like("sk-"), "nothing behind it");
    assert!(!looks_like("sk-short"), "too short");
    assert!(!looks_like(
        "ghp_ has spaces inside it 0123456789012345678901"
    ));
    assert!(!looks_like("AKIA with spaces and all"));
    assert!(!looks_like(
        "sk-not-a-real-key-for-tests-0123456789 and then some"
    ));
    assert!(!looks_like(""));
    assert!(
        !looks_like(&format!("{} with a space", jwt("{}"))),
        "a JWT followed by words is not a token"
    );
    assert!(
        !looks_like("skeleton-key-of-the-castle-0123456789"),
        "an exact sk-, not just its letters"
    );
}

#[test]
fn what_a_person_copies_every_day_is_not_a_token() {
    for text in [
        "https://example.test/path",
        "someone@example.test",
        "7ab3f6de-1c4b-4f5e-8a2d-9f0e1b2c3d4e",
        "d41d8cd98f00b204e9800998ecf8427e",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789abcdefghijklmnop",
        "AKIA",
        "skater",
    ] {
        assert!(!looks_like(text), "{text}");
    }
}

#[test]
fn base64url_decodes_with_or_without_padding() {
    assert_eq!(base64url("aGVsbG8").as_deref(), Some(&b"hello"[..]));
    assert_eq!(base64url("aGVsbG8=").as_deref(), Some(&b"hello"[..]));
    assert_eq!(base64url("aGk").as_deref(), Some(&b"hi"[..]));
    assert_eq!(
        base64url("_w").as_deref(),
        Some(&[0xff][..]),
        "url alphabet"
    );
    assert_eq!(
        base64url("/w").as_deref(),
        Some(&[0xff][..]),
        "and the classic one"
    );
    assert_eq!(base64url("-w").as_deref(), Some(&[0xfb][..]));
    assert_eq!(base64url("+w").as_deref(), Some(&[0xfb][..]));
    assert_eq!(base64url(""), Some(Vec::new()));
    assert_eq!(base64url("a b"), None);
    assert_eq!(base64url("ñ"), None);
}

#[test]
fn dots_without_substance_are_not_a_jwt_either() {
    assert!(claims_of("").is_none());
    assert!(claims_of(".").is_none());
    assert!(claims_of("..").is_none());
    assert!(claims_of("...").is_none());
    assert!(!looks_like(".."));
    assert!(!looks_like("..."));
}

#[test]
fn an_expiry_far_in_the_past_or_the_future_is_still_just_a_comparison() {
    let long_expired = claims_of(&jwt(r#"{"exp":0}"#)).expect("reads");
    assert_eq!(long_expired.expired_by(1_700_000_000), Some(true));

    let far_future = claims_of(&jwt(r#"{"exp":253402300799}"#)).expect("reads");
    assert_eq!(far_future.expired_by(1_700_000_000), Some(false));
}

#[test]
fn an_expiry_of_the_wrong_json_type_is_not_guessed_at() {
    let quoted = claims_of(&jwt(r#"{"exp":"1700000000"}"#)).expect("reads");
    assert_eq!(
        quoted.expires_at(),
        None,
        "a quoted number is text, not the expiry"
    );
    let fractional = claims_of(&jwt(r#"{"exp":1700000000.5}"#)).expect("reads");
    assert_eq!(
        fractional.expires_at(),
        None,
        "a fractional expiry is not read as a whole second either"
    );
}
