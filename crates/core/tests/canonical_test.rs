use agent_test_guard_core::canonical::{
    canonicalize_json_str, canonicalize_value, to_canonical_string, to_canonical_vec,
    CanonicalError,
};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct LogMeta {
    zone: u32,
    cluster: String,
}

#[derive(Serialize)]
struct AuditRecord {
    service: String,
    retries: u32,
    active: bool,
    meta: LogMeta,
}

#[test]
fn test_lexicographical_key_sorting_when_nested_and_utf16_code_units() {
    // Given: nested object with keys ordered differently in UTF-8 vs UTF-16
    let input = json!({
        "\u{20ac}": "euro",
        "\u{1f600}": "emoji",
        "10": 1,
        "2": 2,
        "z": {"b": 2, "a": 1},
        "a": {"d": 4, "c": 3}
    });

    // When: canonicalizing value
    let result = canonicalize_value(&input).expect("canonicalize_value should succeed");

    // Then: keys sorted by UTF-16 code units (10 < 2, a < z, \u20ac < \ud83d\ude00)
    let expected = r#"{"10":1,"2":2,"a":{"c":3,"d":4},"z":{"a":1,"b":2},"€":"euro","😀":"emoji"}"#;
    assert_eq!(result, expected);
}

#[test]
fn test_whitespace_removal_when_multiline_and_spaced_json() {
    // Given: JSON string with newlines, spaces, tabs, and carriage returns
    let raw = "{\r\n  \"b\":  [ 1 ,  \n 2 ] ,\t\n \"a\" : \"hello world\" \n}";

    // When: canonicalizing raw JSON string
    let result = canonicalize_json_str(raw).expect("canonicalize_json_str should succeed");

    // Then: all formatting whitespace outside string literals is stripped
    assert_eq!(result, r#"{"a":"hello world","b":[1,2]}"#);
}

#[test]
fn test_number_formatting_when_floats_and_integers_processed() {
    // Given: integer-valued float, negative zero, decimals, and exponents
    let input = json!({
        "dec": 1.5,
        "exp": 1e21,
        "int": 42.0,
        "neg_zero": -0.0,
        "small": 1e-7
    });

    // When: converting to canonical string
    let result = to_canonical_string(&input).expect("to_canonical_string should succeed");

    // Then: formatted per RFC 8785 (42.0 -> 42, -0.0 -> 0, exponential lowercase)
    let expected = r#"{"dec":1.5,"exp":1e+21,"int":42,"neg_zero":0,"small":1e-7}"#;
    assert_eq!(result, expected);
}

#[test]
fn test_minimal_escaping_when_control_chars_and_utf8_present() {
    // Given: strings with required escapes, forward slashes, Cyrillic, and emoji
    let input = json!({
        "controls": "quote:\" backslash:\\ tab:\t newline:\n",
        "unescaped": "a/b/c Привет мир 🛡️"
    });

    // When: canonicalizing JSON value
    let result = canonicalize_value(&input).expect("canonicalize_value should succeed");

    // Then: only quotes, backslashes, controls escaped; slashes and UTF-8 untouched
    let expected = "{\"controls\":\"quote:\\\" backslash:\\\\ tab:\\t newline:\\n\",\"unescaped\":\"a/b/c Привет мир 🛡️\"}";
    assert_eq!(result, expected);
}

#[test]
fn test_array_ordering_when_elements_unordered() {
    // Given: array with elements in deliberate order containing objects
    let input = json!([
        3,
        1,
        2,
        {"y": 2, "x": 1},
        {"b": 4, "a": 3}
    ]);

    // When: canonicalizing array value
    let result = canonicalize_value(&input).expect("canonicalize_value should succeed");

    // Then: array elements retain order while internal object keys are sorted
    assert_eq!(result, r#"[3,1,2,{"x":1,"y":2},{"a":3,"b":4}]"#);
}

#[test]
fn test_error_handling_when_invalid_json_or_non_finite_float() {
    // Given: malformed JSON string, NaN, and Infinity floats
    let malformed = r#"{"incomplete": "#;
    let nan_val = f64::NAN;
    let inf_val = f64::INFINITY;

    // When: canonicalization is attempted on invalid inputs
    let res_json: Result<String, CanonicalError> = canonicalize_json_str(malformed);
    let res_nan: Result<String, CanonicalError> = to_canonical_string(&nan_val);
    let res_inf: Result<Vec<u8>, CanonicalError> = to_canonical_vec(&inf_val);

    // Then: all operations return CanonicalError
    assert!(matches!(res_json, Err(_)), "invalid json must yield CanonicalError");
    assert!(matches!(res_nan, Err(_)), "NaN float must yield CanonicalError");
    assert!(matches!(res_inf, Err(_)), "Infinity float must yield CanonicalError");
}

#[test]
fn test_idempotence_when_canonicalized_repeatedly() {
    // Given: JSON string with whitespace, unsorted keys, and nested structures
    let raw = " { \"z\" : 1 , \n \"a\" : { \"y\" : 2 , \"x\" : 1 } } ";

    // When: canonicalized once and then canonicalized again
    let first = canonicalize_json_str(raw).expect("first canonicalization should succeed");
    let second = canonicalize_json_str(&first).expect("second canonicalization should succeed");
    let val: serde_json::Value = serde_json::from_str(&first).expect("valid json value");
    let third = canonicalize_value(&val).expect("canonicalize_value should succeed");

    // Then: canonicalization is idempotent across string and value representations
    assert_eq!(first, r#"{"a":{"x":1,"y":2},"z":1}"#);
    assert_eq!(first, second);
    assert_eq!(second, third);
}

#[test]
fn test_typed_struct_serialization_when_string_and_vec_invoked() {
    // Given: a strongly-typed struct implementing Serialize
    let record = AuditRecord {
        service: "auth-guard".to_string(),
        retries: 3,
        active: true,
        meta: LogMeta {
            zone: 42,
            cluster: "prod-us".to_string(),
        },
    };

    // When: serializing to canonical string and canonical vec
    let canon_str = to_canonical_string(&record).expect("to_canonical_string should succeed");
    let canon_vec = to_canonical_vec(&record).expect("to_canonical_vec should succeed");

    // Then: outputs match RFC 8785 representation and byte vector matches string bytes
    let expected = r#"{"active":true,"meta":{"cluster":"prod-us","zone":42},"retries":3,"service":"auth-guard"}"#;
    assert_eq!(canon_str, expected);
    assert_eq!(canon_vec, expected.as_bytes());
}
