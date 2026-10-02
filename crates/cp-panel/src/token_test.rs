use super::{ROWS_AT_MOST, Said, rows_in, said_of};

const SECOND: i64 = 1_000;
const HOUR: i64 = 3_600 * SECOND;

const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

fn segment(said: &str) -> String {
    let mut out = String::new();
    for chunk in said.as_bytes().chunks(3) {
        let mut buffer = 0u32;
        for (at, byte) in chunk.iter().enumerate() {
            buffer |= u32::from(*byte) << (16 - 8 * at);
        }
        for at in 0..=chunk.len() {
            let index = (buffer >> (18 - 6 * at)) & 0x3F;
            out.push(char::from(ALPHABET[index as usize]));
        }
    }
    out
}

fn jwt(payload: &str) -> String {
    format!(
        "{}.{}.firma",
        segment(r#"{"alg":"HS256","typ":"JWT"}"#),
        segment(payload)
    )
}

#[test]
fn something_that_is_not_a_token_has_nothing_to_show() {
    assert_eq!(said_of("sk-ant-api03-no-soy-un-jwt", 0, false), None);
    assert_eq!(said_of("", 0, false), None);
    assert_eq!(
        rows_in("sk-ant-api03-no-soy-un-jwt"),
        0,
        "una clave de API no tiene nada que descifrar, asi que no pide ni una fila"
    );
}

#[test]
fn a_token_says_who_it_is_for_and_who_signed_it() {
    let said = said_of(&jwt(r#"{"sub":"rodrigo","iss":"cloudflare"}"#), 0, false)
        .expect("un jwt se descifra");
    assert_eq!(said.who, "rodrigo · cloudflare");
}

#[test]
fn a_token_with_only_an_issuer_does_not_leave_a_dangling_separator() {
    let said = said_of(&jwt(r#"{"iss":"cloudflare"}"#), 0, false).expect("un jwt");
    assert_eq!(said.who, "cloudflare");
}

#[test]
fn a_live_token_says_how_long_it_has_left() {
    let said = said_of(&jwt(r#"{"exp":7200}"#), HOUR, false).expect("un jwt");
    assert_eq!(said.life, "caduca en 1 h");
}

#[test]
fn a_dead_token_says_how_long_ago_it_died() {
    let said = said_of(&jwt(r#"{"exp":3600}"#), 3 * HOUR, false).expect("un jwt");
    assert_eq!(
        said.life, "hace 2 h",
        "the badge already shouts CADUCADO, so the line says only how long ago"
    );
}

#[test]
fn a_token_without_an_expiry_promises_no_countdown() {
    let said = said_of(&jwt(r#"{"sub":"rodrigo"}"#), 0, false).expect("un jwt");
    assert_eq!(said.life, "");
}

#[test]
fn the_registered_claims_read_in_the_order_people_expect() {
    let said = said_of(
        &jwt(r#"{"scope":"read","exp":7200,"sub":"rodrigo","iss":"cloudflare"}"#),
        0,
        false,
    )
    .expect("un jwt");
    assert_eq!(said.names, "iss\nsub\nexp\nscope");
}

#[test]
fn the_two_columns_always_have_the_same_number_of_lines() {
    let payload = r#"{"iss":"a","sub":"b","aud":"c","exp":1,"iat":2,"nbf":3,"jti":"d",
        "scope":"read write","role":"admin","email":"yo@ejemplo.cl","tier":"pro",
        "deep":{"one":1},"list":[1,2],"extra":"x","more":"y"}"#;
    let said = said_of(&jwt(payload), 0, false).expect("un jwt");
    assert_eq!(
        said.names.lines().count(),
        said.values.lines().count(),
        "las dos columnas se dibujan lado a lado, asi que una fila de mas en una las desalinea todas"
    );
}

#[test]
fn a_payload_longer_than_the_card_says_how_many_it_left_out() {
    let mut payload = String::from("{");
    for one in 0..20 {
        payload.push_str(&format!("\"k{one}\":\"v\","));
    }
    payload.push_str("\"sub\":\"rodrigo\"}");
    let said = said_of(&jwt(&payload), 0, false).expect("un jwt");
    assert_eq!(said.names.lines().count(), ROWS_AT_MOST + 1);
    assert!(
        said.names.ends_with("y más"),
        "la ultima fila dice cuantas no caben: {}",
        said.names
    );
    assert!(said.values.ends_with("+9"), "{}", said.values);
}

#[test]
fn a_value_with_line_breaks_is_flattened_so_the_columns_stay_paired() {
    let said = said_of(&jwt("{\"note\":\"dos\\nlineas\"}"), 0, false).expect("un jwt");
    assert_eq!(said.values, "dos lineas");
}

#[test]
fn a_nested_value_is_marked_and_not_spilled_into_the_cell() {
    let said = said_of(
        &jwt(r#"{"deep":{"one":1,"two":2},"list":[1,2,3]}"#),
        0,
        false,
    )
    .expect("un jwt");
    assert_eq!(said.values, "{ … }\n[ … ]");
}

#[test]
fn the_dates_read_as_a_distance_and_not_as_a_number_of_seconds() {
    let said = said_of(&jwt(r#"{"iat":3600,"exp":10800}"#), 2 * HOUR, false).expect("un jwt");
    assert_eq!(said.values, "en 1 h\nhace 1 h");
}

#[test]
fn the_row_count_matches_the_table_the_card_will_draw() {
    let payload = r#"{"iss":"a","sub":"b","exp":1,"scope":"read"}"#;
    let said = said_of(&jwt(payload), 0, false).expect("un jwt");
    assert_eq!(
        rows_in(&jwt(payload)),
        said.names.lines().count(),
        "el modelo reserva alto por esta cuenta, asi que si miente la tarjeta se corta"
    );
}

#[test]
fn english_says_it_in_english() {
    let said = said_of(&jwt(r#"{"exp":7200,"iat":0}"#), HOUR, true).expect("un jwt");
    assert_eq!(said.life, "expires in 1 h");
    assert_eq!(said.values, "in 1 h\n1 h ago");
}

#[test]
fn a_said_with_nothing_in_it_is_what_default_gives() {
    assert_eq!(
        Said::default(),
        Said {
            who: String::new(),
            life: String::new(),
            names: String::new(),
            values: String::new(),
        }
    );
}

#[test]
fn the_token_from_the_screen_probe_reads() {
    let real = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJjbG91ZGZsYXJlIiwic3ViIjoicm9kcmlnbyIsImF1ZCI6ImNvcHlwYXN0ZSIsImV4cCI6MTc5MDk1MjkxNiwiaWF0IjoxNzkwOTQ1MTE2LCJzY29wZSI6InJlYWQgd3JpdGUiLCJyb2xlIjoiYWRtaW4ifQ.bWFyY2EtZGUtcHJ1ZWJhLW5vLXZhbGlkYQ";
    let said = said_of(real, 1_790_945_716_000, false).expect("el token de la pantalla");
    assert_eq!(said.who, "rodrigo \u{b7} cloudflare");
    assert_eq!(said.names, "iss\nsub\naud\nexp\niat\nscope\nrole");
    assert_eq!(said.life, "caduca en 2 h");
}

#[test]
fn a_claim_with_nothing_in_it_does_not_take_a_line() {
    let said =
        said_of(&jwt(r#"{"iss":"","sub":"rodrigo","aud":"   "}"#), 0, false).expect("un jwt");
    assert_eq!(
        said.names, "sub",
        "una clave vacia ocupa una linea para no decir nada, y la tarjeta se mide por lineas"
    );
    assert_eq!(said.values, "rodrigo");
}

#[test]
fn a_claim_with_an_absurd_moment_does_not_bring_the_panel_down() {
    for seconds in [
        i64::MIN,
        i64::MAX,
        -9_223_372_036_854_775,
        9_223_372_036_854_775,
    ] {
        let payload = format!(r#"{{"exp":{seconds},"iat":{seconds}}}"#);
        let said = said_of(&jwt(&payload), 1_760_000_000_000, false)
            .expect("a token with a number nobody meant is still a token");
        assert!(!said.life.is_empty());
        assert!(!said.values.is_empty());
    }
}
