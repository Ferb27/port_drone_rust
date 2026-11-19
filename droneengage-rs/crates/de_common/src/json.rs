// Ported from droneengage_common@5fb11c2: helpers/helpers.cpp (validateField)
//
// nlohmann::json keeps three number kinds. A parsed non-negative integer is
// `number_unsigned`, a parsed negative integer is `number_integer`, anything
// with a fraction/exponent is `number_float`. C++ code checks these kinds, so
// we mirror the classification on top of serde_json::Value.

use serde_json::Value;

/// nlohmann::json::value_t equivalent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonType {
    Null,
    Object,
    Array,
    String,
    Boolean,
    NumberInteger,
    NumberUnsigned,
    NumberFloat,
}

/// The nlohmann value_t a parsed value would have.
pub fn json_type(v: &Value) -> JsonType {
    match v {
        Value::Null => JsonType::Null,
        Value::Bool(_) => JsonType::Boolean,
        Value::String(_) => JsonType::String,
        Value::Array(_) => JsonType::Array,
        Value::Object(_) => JsonType::Object,
        Value::Number(n) => {
            if n.is_u64() {
                JsonType::NumberUnsigned
            } else if n.is_i64() {
                JsonType::NumberInteger
            } else {
                JsonType::NumberFloat
            }
        }
    }
}

/// True if `message[field]` exists and has exactly type `t` (C++ validateField).
pub fn validate_field(message: &Value, field: &str, t: JsonType) -> bool {
    message
        .get(field)
        .map(|v| json_type(v) == t)
        .unwrap_or(false)
}

/// nlohmann `get<int>()` on any number kind (floats truncate, out of range wraps like a C cast).
pub fn as_cpp_int(v: &Value) -> Option<i32> {
    match v {
        Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Some(u as i32)
            } else if let Some(i) = n.as_i64() {
                Some(i as i32)
            } else {
                n.as_f64().map(|f| f as i32)
            }
        }
        _ => None,
    }
}

/// Serialize the way C++ `Json_de::dump()` does: compact, keys sorted.
/// serde_json without `preserve_order` already sorts object keys (BTreeMap).
pub fn dump(v: &Value) -> String {
    // to_string on a Value cannot fail (keys are strings).
    serde_json::to_string(v).unwrap_or_default()
}

/// Parse the JSON text that precedes the first NUL byte, like nlohmann
/// parsing a `const char*` (binary messages are "json\0payload").
pub fn parse_c_string(raw: &[u8]) -> Result<Value, serde_json::Error> {
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    serde_json::from_slice(&raw[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn number_kinds_match_nlohmann() {
        let v: Value = serde_json::from_str(r#"{"u":5,"i":-5,"f":5.0,"p":4294967295}"#).unwrap();
        assert!(validate_field(&v, "u", JsonType::NumberUnsigned));
        assert!(validate_field(&v, "i", JsonType::NumberInteger));
        assert!(validate_field(&v, "f", JsonType::NumberFloat));
        assert!(validate_field(&v, "p", JsonType::NumberUnsigned));
        assert!(!validate_field(&v, "missing", JsonType::Null));
        assert_eq!(as_cpp_int(&v["p"]), Some(-1));
    }

    #[test]
    fn dump_sorts_keys_and_keeps_float_suffix() {
        let v = json!({"ty":"g","GU":"k","ms":{"A":15002.0,"b":null},"mt":1037});
        assert_eq!(
            dump(&v),
            r#"{"GU":"k","ms":{"A":15002.0,"b":null},"mt":1037,"ty":"g"}"#
        );
    }

    #[test]
    fn parse_stops_at_nul() {
        let v = parse_c_string(b"{\"a\":1}\0\x01\x02garbage").unwrap();
        assert_eq!(v, json!({"a":1}));
    }
}
