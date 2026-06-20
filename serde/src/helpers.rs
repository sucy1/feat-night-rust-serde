//! Helper functions for common serde customization patterns.
//!
//! This module provides reusable helpers that reduce the need for handwritten
//! custom [`Serialize`]/[`Deserialize`] implementations in frequently
//! encountered scenarios.
//!
//! # Examples
//!
//! ```
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Debug, PartialEq, Serialize, Deserialize)]
//! struct Example {
//!     #[serde(with = "serde::helpers::with_default")]
//!     port: u16,
//!
//!     #[serde(
//!         with = "serde::helpers::skip_empty",
//!         skip_serializing_if = "serde::helpers::skip_empty::is_empty"
//!     )]
//!     tags: Vec<String>,
//!
//!     #[serde(
//!         with = "serde::helpers::skip_empty",
//!         skip_serializing_if = "serde::helpers::skip_empty::is_empty"
//!     )]
//!     note: String,
//! }
//! ```

use crate::lib::*;

use crate::de::{Deserialize, Deserializer};
use crate::ser::{Serialize, Serializer};

////////////////////////////////////////////////////////////////////////////////
// with_default
////////////////////////////////////////////////////////////////////////////////

/// Deserialize a field, falling back to [`Default::default()`] when the field
/// is missing or `null`.
///
/// Use with `#[serde(with = "serde::helpers::with_default")]`. Unlike the
/// standard `#[serde(default)]` attribute, this helper also converts an
/// explicit `null` value (where supported by the format) into the default.
///
/// # Examples
///
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Config {
///     #[serde(with = "serde::helpers::with_default")]
///     max_connections: u32,
///
///     #[serde(with = "serde::helpers::with_default")]
///     name: String,
/// }
/// ```
pub mod with_default {
    use super::*;

    /// Serialize a value using its ordinary [`Serialize`] implementation.
    ///
    /// This is provided so that `#[serde(with = "with_default")]` works both
    /// ways without needing to pair it with a separate `serialize_with`.
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        value.serialize(serializer)
    }

    /// Deserialize a value, substituting [`Default::default()`] when the input
    /// is missing or represents `null`.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de> + Default,
    {
        Option::<T>::deserialize(deserializer).map(Option::unwrap_or_default)
    }
}

////////////////////////////////////////////////////////////////////////////////
// skip_empty
////////////////////////////////////////////////////////////////////////////////

/// Skip serialization and/or treat as "empty" common types that have a natural
/// empty state (`String`, `Vec`, slices).
///
/// Typical usage:
///
/// ```
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Debug, PartialEq, Serialize, Deserialize)]
/// struct Request {
///     #[serde(
///         with = "serde::helpers::skip_empty",
///         skip_serializing_if = "serde::helpers::skip_empty::is_empty"
///     )]
///     query: String,
///
///     #[serde(
///         with = "serde::helpers::skip_empty",
///         skip_serializing_if = "serde::helpers::skip_empty::is_empty"
///     )]
///     filters: Vec<String>,
/// }
/// ```
pub mod skip_empty {
    use super::*;

    /// Types that have a semantically meaningful "empty" state.
    pub trait IsEmpty {
        /// Returns `true` if the value should be considered empty.
        fn is_empty(&self) -> bool;
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    impl IsEmpty for String {
        #[inline]
        fn is_empty(&self) -> bool {
            String::is_empty(self)
        }
    }

    impl IsEmpty for str {
        #[inline]
        fn is_empty(&self) -> bool {
            str::is_empty(self)
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    impl<T> IsEmpty for Vec<T> {
        #[inline]
        fn is_empty(&self) -> bool {
            Vec::is_empty(self)
        }
    }

    impl<T> IsEmpty for [T] {
        #[inline]
        fn is_empty(&self) -> bool {
            <[T]>::is_empty(self)
        }
    }

    impl<T: IsEmpty + ?Sized> IsEmpty for &T {
        #[inline]
        fn is_empty(&self) -> bool {
            (**self).is_empty()
        }
    }

    /// Predicate suitable for `#[serde(skip_serializing_if = "...")]`.
    ///
    /// Returns `true` when `value` reports itself as empty through the
    /// [`IsEmpty`] trait.
    #[inline]
    pub fn is_empty<T: IsEmpty>(value: &T) -> bool {
        IsEmpty::is_empty(value)
    }

    /// Serialize a value using its ordinary [`Serialize`] implementation.
    pub fn serialize<S, T>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Serialize,
    {
        value.serialize(serializer)
    }

    /// Deserialize a value using its ordinary [`Deserialize`] implementation.
    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        T::deserialize(deserializer)
    }
}

////////////////////////////////////////////////////////////////////////////////
// rename_all
////////////////////////////////////////////////////////////////////////////////

/// Convert field-name casing between `snake_case` and `camelCase`.
///
/// These helpers are useful when writing a manual [`Serialize`] or
/// [`Deserialize`] implementation and want to apply a consistent rename rule
/// across fields without using `#[derive(Serialize)]`.
///
/// # Examples
///
/// ```
/// assert_eq!(serde::helpers::rename_all::to_camel_case("user_name"), "userName");
/// assert_eq!(serde::helpers::rename_all::to_snake_case("userName"), "user_name");
/// ```
pub mod rename_all {
    use super::*;

    /// Convert a `snake_case` identifier to `camelCase`.
    ///
    /// Leading and trailing underscores are preserved. Runs of uppercase
    /// letters (e.g. acronyms) are lower-cased as a group.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(serde::helpers::rename_all::to_camel_case("hello_world"), "helloWorld");
    /// assert_eq!(serde::helpers::rename_all::to_camel_case("a"), "a");
    /// assert_eq!(serde::helpers::rename_all::to_camel_case(""), "");
    /// assert_eq!(serde::helpers::rename_all::to_camel_case("_leading"), "_leading");
    /// assert_eq!(serde::helpers::rename_all::to_camel_case("xml_http_request"), "xmlHttpRequest");
    /// ```
    pub fn to_camel_case(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut capitalize_next = false;

        for (i, ch) in input.char_indices() {
            if ch == '_' {
                if i == 0 || out.is_empty() {
                    out.push(ch);
                } else {
                    capitalize_next = true;
                }
            } else if capitalize_next {
                for upper in ch.to_uppercase() {
                    out.push(upper);
                }
                capitalize_next = false;
            } else {
                out.push(ch);
            }
        }

        if capitalize_next {
            out.push('_');
        }

        out
    }

    /// Convert a `camelCase` or `PascalCase` identifier to `snake_case`.
    ///
    /// Uppercase letters are preceded by an underscore (except at the start)
    /// and lower-cased. Consecutive uppercase letters (acronyms) are kept as a
    /// single group until the final uppercase letter followed by a lowercase
    /// letter, which begins a new word.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("helloWorld"), "hello_world");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("HelloWorld"), "hello_world");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("a"), "a");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case(""), "");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("XMLHttpRequest"), "xml_http_request");
    /// ```
    pub fn to_snake_case(input: &str) -> String {
        let mut out = String::with_capacity(input.len());
        let mut prev_is_upper = false;

        for (i, ch) in input.char_indices() {
            if ch.is_uppercase() {
                if i > 0 && !prev_is_upper {
                    out.push('_');
                } else if i > 0 && prev_is_upper {
                    let next_is_lower = input[i..].chars().nth(1).map(|c| c.is_lowercase()).unwrap_or(false);
                    if next_is_lower {
                        out.push('_');
                    }
                }
                for lower in ch.to_lowercase() {
                    out.push(lower);
                }
                prev_is_upper = true;
            } else {
                out.push(ch);
                prev_is_upper = false;
            }
        }

        out
    }
}

////////////////////////////////////////////////////////////////////////////////
// Tests
////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_default_returns_default_on_none() {
        use serde_test::{assert_de_tokens, Token};

        #[derive(Debug, PartialEq, Deserialize)]
        struct S {
            #[serde(with = "with_default")]
            x: u32,
            #[serde(with = "with_default")]
            y: String,
        }

        assert_de_tokens(
            &S { x: 0, y: String::new() },
            &[
                Token::Struct { name: "S", len: 2 },
                Token::Str("x"),
                Token::None,
                Token::Str("y"),
                Token::None,
                Token::StructEnd,
            ],
        );
    }

    #[test]
    fn with_default_preserves_values() {
        use serde_test::{assert_tokens, Token};

        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct S {
            #[serde(with = "with_default")]
            x: u32,
            #[serde(with = "with_default")]
            y: String,
        }

        assert_tokens(
            &S { x: 42, y: "hi".into() },
            &[
                Token::Struct { name: "S", len: 2 },
                Token::Str("x"),
                Token::U32(42),
                Token::Str("y"),
                Token::Str("hi"),
                Token::StructEnd,
            ],
        );
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_is_empty_string_and_vec() {
        assert!(skip_empty::is_empty(&String::new()));
        assert!(!skip_empty::is_empty(&String::from("x")));

        let empty: Vec<u32> = vec![];
        let full: Vec<u32> = vec![1];
        assert!(skip_empty::is_empty(&empty));
        assert!(!skip_empty::is_empty(&full));
    }

    #[test]
    fn skip_empty_is_empty_str_and_slice() {
        assert!(skip_empty::is_empty(""));
        assert!(!skip_empty::is_empty("a"));

        let empty: &[u32] = &[];
        let full: &[u32] = &[1, 2];
        assert!(skip_empty::is_empty(empty));
        assert!(!skip_empty::is_empty(full));
    }

    #[test]
    fn to_camel_case_basic() {
        assert_eq!(rename_all::to_camel_case("hello_world"), "helloWorld");
        assert_eq!(rename_all::to_camel_case("a_b_c"), "aBC");
        assert_eq!(rename_all::to_camel_case("single"), "single");
        assert_eq!(rename_all::to_camel_case(""), "");
    }

    #[test]
    fn to_camel_case_underscores() {
        assert_eq!(rename_all::to_camel_case("_leading"), "_leading");
        assert_eq!(rename_all::to_camel_case("__leading"), "__leading");
        assert_eq!(rename_all::to_camel_case("a_"), "a_");
        assert_eq!(rename_all::to_camel_case("a__b"), "aB");
    }

    #[test]
    fn to_snake_case_basic() {
        assert_eq!(rename_all::to_snake_case("helloWorld"), "hello_world");
        assert_eq!(rename_all::to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(rename_all::to_snake_case("aBC"), "a_b_c");
        assert_eq!(rename_all::to_snake_case("single"), "single");
        assert_eq!(rename_all::to_snake_case(""), "");
    }

    #[test]
    fn to_snake_case_acronyms() {
        assert_eq!(rename_all::to_snake_case("XMLHttpRequest"), "xml_http_request");
        assert_eq!(rename_all::to_snake_case("HTTPSConnection"), "https_connection");
        assert_eq!(rename_all::to_snake_case("myURLParser"), "my_url_parser");
    }

    #[test]
    fn roundtrip_rename_case() {
        let words = &["hello_world", "user_name", "xml_http_request", "a_b_c"];
        for w in words {
            let camel = rename_all::to_camel_case(w);
            let back = rename_all::to_snake_case(&camel);
            assert_eq!(&back, w);
        }
    }
}
