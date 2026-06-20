use serde::{Deserialize, Serialize};
use serde_test::{assert_de_tokens, assert_ser_tokens, assert_tokens, Token};

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
fn with_default_roundtrip() {
    assert_tokens(
        &WithDefaultS {
            x: 42,
            y: "hello".into(),
        },
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
}

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
fn skip_empty_skips_when_empty() {
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
fn skip_empty_includes_when_present() {
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
