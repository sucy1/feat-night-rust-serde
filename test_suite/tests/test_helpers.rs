use serde_derive::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_de_tokens_error, assert_ser_tokens, assert_tokens, Token};

// =============================================================================
// with_default: null → Default::default()
// =============================================================================

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct WithDefaultS {
    #[serde(with = "serde::helpers::with_default")]
    x: u32,
    #[serde(with = "serde::helpers::with_default")]
    y: String,
}

#[test]
fn with_default_null_becomes_default() {
    assert_de_tokens(
        &WithDefaultS {
            x: 0,
            y: String::new(),
        },
        &[
            Token::Struct {
                name: "WithDefaultS",
                len: 2,
            },
            Token::Str("x"),
            Token::None,
            Token::Str("y"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

#[test]
fn with_default_some_value_passes_through() {
    assert_de_tokens(
        &WithDefaultS {
            x: 99,
            y: "hello".into(),
        },
        &[
            Token::Struct {
                name: "WithDefaultS",
                len: 2,
            },
            Token::Str("x"),
            Token::Some,
            Token::U32(99),
            Token::Str("y"),
            Token::Some,
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn with_default_roundtrip() {
    let value = WithDefaultS {
        x: 42,
        y: "hello".into(),
    };

    assert_ser_tokens(
        &value,
        &[
            Token::Struct {
                name: "WithDefaultS",
                len: 2,
            },
            Token::Str("x"),
            Token::U32(42),
            Token::Str("y"),
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "WithDefaultS",
                len: 2,
            },
            Token::Str("x"),
            Token::Some,
            Token::U32(42),
            Token::Str("y"),
            Token::Some,
            Token::Str("hello"),
            Token::StructEnd,
        ],
    );
}

// =============================================================================
// with_default: missing field requires #[serde(default)] in addition
// =============================================================================

#[derive(Debug, PartialEq, Deserialize)]
struct WithDefaultAndMissingS {
    #[serde(with = "serde::helpers::with_default", default)]
    port: u16,
    #[serde(with = "serde::helpers::with_default", default)]
    name: String,
}

#[test]
fn with_default_missing_field_uses_default_when_combined_with_serde_default() {
    assert_de_tokens(
        &WithDefaultAndMissingS {
            port: 0,
            name: String::new(),
        },
        &[
            Token::Struct {
                name: "WithDefaultAndMissingS",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn with_default_missing_field_overrides_with_provided_value() {
    assert_de_tokens(
        &WithDefaultAndMissingS {
            port: 8080,
            name: "api".into(),
        },
        &[
            Token::Struct {
                name: "WithDefaultAndMissingS",
                len: 2,
            },
            Token::Str("port"),
            Token::Some,
            Token::U16(8080),
            Token::Str("name"),
            Token::Some,
            Token::Str("api"),
            Token::StructEnd,
        ],
    );
}

// =============================================================================
// with_default: type mismatch produces an error (not silently default)
// =============================================================================

#[derive(Debug, PartialEq, Deserialize)]
struct WithDefaultTypeMismatchS {
    #[serde(with = "serde::helpers::with_default")]
    x: u32,
}

#[test]
fn with_default_type_mismatch_errors() {
    assert_de_tokens_error::<WithDefaultTypeMismatchS>(
        &[
            Token::Struct {
                name: "WithDefaultTypeMismatchS",
                len: 1,
            },
            Token::Str("x"),
            Token::Some,
            Token::Str("not_a_number"),
            Token::StructEnd,
        ],
        "invalid type: string \"not_a_number\", expected u32",
    );
}

// =============================================================================
// with_default: various Default types
// =============================================================================

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct WithDefaultVariousTypesS {
    #[serde(with = "serde::helpers::with_default")]
    flag: bool,
    #[serde(with = "serde::helpers::with_default")]
    items: Vec<u32>,
    #[serde(with = "serde::helpers::with_default")]
    maybe: Option<String>,
}

#[test]
fn with_default_null_gives_various_defaults() {
    assert_de_tokens(
        &WithDefaultVariousTypesS {
            flag: false,
            items: vec![],
            maybe: None,
        },
        &[
            Token::Struct {
                name: "WithDefaultVariousTypesS",
                len: 3,
            },
            Token::Str("flag"),
            Token::None,
            Token::Str("items"),
            Token::None,
            Token::Str("maybe"),
            Token::None,
            Token::StructEnd,
        ],
    );
}

#[test]
fn with_default_values_preserved_for_various_types() {
    let value = WithDefaultVariousTypesS {
        flag: true,
        items: vec![1, 2],
        maybe: Some("yes".into()),
    };

    assert_ser_tokens(
        &value,
        &[
            Token::Struct {
                name: "WithDefaultVariousTypesS",
                len: 3,
            },
            Token::Str("flag"),
            Token::Bool(true),
            Token::Str("items"),
            Token::Seq { len: Some(2) },
            Token::U32(1),
            Token::U32(2),
            Token::SeqEnd,
            Token::Str("maybe"),
            Token::Some,
            Token::Str("yes"),
            Token::StructEnd,
        ],
    );

    assert_de_tokens(
        &value,
        &[
            Token::Struct {
                name: "WithDefaultVariousTypesS",
                len: 3,
            },
            Token::Str("flag"),
            Token::Some,
            Token::Bool(true),
            Token::Str("items"),
            Token::Some,
            Token::Seq { len: Some(2) },
            Token::U32(1),
            Token::U32(2),
            Token::SeqEnd,
            Token::Str("maybe"),
            Token::Some,
            Token::Some,
            Token::Str("yes"),
            Token::StructEnd,
        ],
    );
}

// =============================================================================
// skip_empty: serialization skips empty, includes non-empty
// =============================================================================

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct SkipEmptyS {
    #[serde(
        with = "serde::helpers::skip_empty",
        skip_serializing_if = "serde::helpers::skip_empty::is_empty"
    )]
    name: String,
    #[serde(
        with = "serde::helpers::skip_empty",
        skip_serializing_if = "serde::helpers::skip_empty::is_empty"
    )]
    items: Vec<u32>,
}

#[test]
fn skip_empty_both_empty() {
    assert_ser_tokens(
        &SkipEmptyS {
            name: String::new(),
            items: vec![],
        },
        &[
            Token::Struct {
                name: "SkipEmptyS",
                len: 0,
            },
            Token::StructEnd,
        ],
    );
}

#[test]
fn skip_empty_both_present() {
    assert_tokens(
        &SkipEmptyS {
            name: "alice".into(),
            items: vec![1, 2, 3],
        },
        &[
            Token::Struct {
                name: "SkipEmptyS",
                len: 2,
            },
            Token::Str("name"),
            Token::Str("alice"),
            Token::Str("items"),
            Token::Seq { len: Some(3) },
            Token::U32(1),
            Token::U32(2),
            Token::U32(3),
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn skip_empty_name_present_items_empty() {
    assert_ser_tokens(
        &SkipEmptyS {
            name: "bob".into(),
            items: vec![],
        },
        &[
            Token::Struct {
                name: "SkipEmptyS",
                len: 1,
            },
            Token::Str("name"),
            Token::Str("bob"),
            Token::StructEnd,
        ],
    );
}

#[test]
fn skip_empty_deserialize_empty_values() {
    assert_de_tokens(
        &SkipEmptyS {
            name: String::new(),
            items: vec![],
        },
        &[
            Token::Struct {
                name: "SkipEmptyS",
                len: 2,
            },
            Token::Str("name"),
            Token::Str(""),
            Token::Str("items"),
            Token::Seq { len: Some(0) },
            Token::SeqEnd,
            Token::StructEnd,
        ],
    );
}

#[test]
fn skip_empty_whitespace_not_empty() {
    assert_ser_tokens(
        &SkipEmptyS {
            name: String::from("  "),
            items: vec![],
        },
        &[
            Token::Struct {
                name: "SkipEmptyS",
                len: 1,
            },
            Token::Str("name"),
            Token::Str("  "),
            Token::StructEnd,
        ],
    );
}

// =============================================================================
// rename_all: camelCase / snake_case conversion
// =============================================================================

#[test]
fn rename_all_to_camel_case() {
    use serde::helpers::rename_all::to_camel_case;
    assert_eq!(to_camel_case("user_name"), "userName");
    assert_eq!(to_camel_case("hello_world_foo"), "helloWorldFoo");
}

#[test]
fn rename_all_to_snake_case() {
    use serde::helpers::rename_all::to_snake_case;
    assert_eq!(to_snake_case("userName"), "user_name");
    assert_eq!(to_snake_case("helloWorldFoo"), "hello_world_foo");
    assert_eq!(to_snake_case("HelloWorld"), "hello_world");
}

#[test]
fn rename_all_roundtrip() {
    use serde::helpers::rename_all::{to_camel_case, to_snake_case};
    let words = &["hello_world", "user_name", "a_b_c", "field1_name2"];
    for w in words {
        let camel = to_camel_case(w);
        let back = to_snake_case(&camel);
        assert_eq!(&back, w);
    }
}
