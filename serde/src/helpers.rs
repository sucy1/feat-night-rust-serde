//! Helper functions for common serde customization patterns.
//!
//! This module provides reusable helpers that reduce the need for handwritten
//! custom [`Serialize`]/[`Deserialize`] implementations in frequently
//! encountered scenarios.
//!
//! # Examples
//!
//! ```ignore
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
/// is `null` in the input.
///
/// Use with `#[serde(with = "serde::helpers::with_default")]`. Unlike the
/// standard `#[serde(default)]` attribute, this helper also converts an
/// explicit `null` value (where supported by the format) into the default.
///
/// # Handling missing fields
///
/// This helper only handles `null` values that are **present** in the input.
/// If the field is **entirely missing** from the input, you must also add
/// `#[serde(default)]` alongside `with`:
///
/// ```ignore
/// #[serde(with = "serde::helpers::with_default", default)]
/// port: u16,
/// ```
///
/// # Examples
///
/// ```ignore
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
/// ```ignore
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
    pub fn is_empty<T: IsEmpty + ?Sized>(value: &T) -> bool {
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

/// Convert field-name casing between `snake_case` and `camelCase`/`PascalCase`.
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
        let mut seen_non_underscore = false;

        for ch in input.chars() {
            if ch == '_' {
                if !seen_non_underscore {
                    out.push(ch);
                } else {
                    capitalize_next = true;
                }
            } else {
                seen_non_underscore = true;
                if capitalize_next {
                    for upper in ch.to_uppercase() {
                        out.push(upper);
                    }
                    capitalize_next = false;
                } else {
                    out.push(ch);
                }
            }
        }

        if capitalize_next {
            out.push('_');
        }

        out
    }

    /// Convert a `camelCase` or `PascalCase` identifier to `snake_case`.
    ///
    /// Every uppercase letter is preceded by an underscore (except at the very
    /// start) and converted to lowercase.
    ///
    /// # Examples
    ///
    /// ```
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("helloWorld"), "hello_world");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("HelloWorld"), "hello_world");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("a"), "a");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case(""), "");
    /// assert_eq!(serde::helpers::rename_all::to_snake_case("UserName"), "user_name");
    /// ```
    pub fn to_snake_case(input: &str) -> String {
        let mut out = String::with_capacity(input.len());

        for (i, ch) in input.char_indices() {
            if ch.is_uppercase() {
                if i > 0 && out.chars().next_back() != Some('_') {
                    out.push('_');
                }
                for lower in ch.to_lowercase() {
                    out.push(lower);
                }
            } else {
                out.push(ch);
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

    // ---- skip_empty: IsEmpty trait edge cases ----

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_is_empty_string_and_vec() {
        assert!(skip_empty::is_empty(&String::new()));
        assert!(!skip_empty::is_empty(&String::from("x")));

        let empty: Vec<u32> = Vec::new();
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

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_whitespace_is_not_empty() {
        assert!(!skip_empty::is_empty(&String::from(" ")));
        assert!(!skip_empty::is_empty(&String::from("\t\n")));
        assert!(!skip_empty::is_empty(&String::from("\u{00a0}")));
    }

    #[test]
    fn skip_empty_str_whitespace_is_not_empty() {
        assert!(!skip_empty::is_empty(" "));
        assert!(!skip_empty::is_empty("\t"));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_ref_delegates() {
        let s = String::from("hello");
        assert!(!skip_empty::is_empty(&s));
        assert!(!skip_empty::is_empty(&&s));

        let empty_s = String::new();
        assert!(skip_empty::is_empty(&empty_s));
        assert!(skip_empty::is_empty(&&empty_s));
    }

    #[test]
    fn skip_empty_ref_str_delegates() {
        let s: &str = "x";
        assert!(!skip_empty::is_empty(s));
        assert!(!skip_empty::is_empty(&s as &dyn skip_empty::IsEmpty));

        let empty: &str = "";
        assert!(skip_empty::is_empty(empty));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_vec_zst() {
        let empty: Vec<()> = Vec::new();
        assert!(skip_empty::is_empty(&empty));

        let full: Vec<()> = vec![(), (), ()];
        assert!(!skip_empty::is_empty(&full));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_vec_single_element() {
        let single: Vec<u8> = vec![0];
        assert!(!skip_empty::is_empty(&single));
    }

    #[test]
    fn skip_empty_slice_single_element() {
        let single: &[i32] = &[42];
        assert!(!skip_empty::is_empty(single));
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn skip_empty_string_special_chars() {
        assert!(!skip_empty::is_empty(&String::from("\0")));
        assert!(!skip_empty::is_empty(&String::from("🦀")));
        assert!(!skip_empty::is_empty(&String::from("0")));
    }

    // ---- rename_all: to_camel_case edge cases ----

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_basic() {
        assert_eq!(rename_all::to_camel_case("hello_world"), "helloWorld");
        assert_eq!(rename_all::to_camel_case("a_b_c"), "aBC");
        assert_eq!(rename_all::to_camel_case("single"), "single");
        assert_eq!(rename_all::to_camel_case(""), "");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_leading_underscores() {
        assert_eq!(rename_all::to_camel_case("_leading"), "_leading");
        assert_eq!(rename_all::to_camel_case("__leading"), "__leading");
        assert_eq!(rename_all::to_camel_case("___a"), "___a");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_trailing_underscore() {
        assert_eq!(rename_all::to_camel_case("a_"), "a_");
        assert_eq!(rename_all::to_camel_case("hello_"), "hello_");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_consecutive_underscores() {
        assert_eq!(rename_all::to_camel_case("a__b"), "aB");
        assert_eq!(rename_all::to_camel_case("a___b"), "aB");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_all_underscores() {
        assert_eq!(rename_all::to_camel_case("___"), "___");
        assert_eq!(rename_all::to_camel_case("_"), "_");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_with_numbers() {
        assert_eq!(rename_all::to_camel_case("field1_name2"), "field1Name2");
        assert_eq!(rename_all::to_camel_case("v2_api_url"), "v2ApiUrl");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_already_camel() {
        assert_eq!(rename_all::to_camel_case("helloWorld"), "helloWorld");
        assert_eq!(rename_all::to_camel_case("aBC"), "aBC");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_single_char() {
        assert_eq!(rename_all::to_camel_case("a"), "a");
        assert_eq!(rename_all::to_camel_case("z"), "z");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_case_uppercase_in_input() {
        assert_eq!(rename_all::to_camel_case("XML_Request"), "XMLRequest");
        assert_eq!(rename_all::to_camel_case("my_URL_parser"), "myURLParser");
    }

    // ---- rename_all: to_snake_case edge cases ----

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_snake_case_basic() {
        assert_eq!(rename_all::to_snake_case("helloWorld"), "hello_world");
        assert_eq!(rename_all::to_snake_case("HelloWorld"), "hello_world");
        assert_eq!(rename_all::to_snake_case("aBC"), "a_b_c");
        assert_eq!(rename_all::to_snake_case("single"), "single");
        assert_eq!(rename_all::to_snake_case(""), "");
        assert_eq!(rename_all::to_snake_case("UserName"), "user_name");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_snake_case_already_snake() {
        assert_eq!(rename_all::to_snake_case("hello_world"), "hello_world");
        assert_eq!(rename_all::to_snake_case("a_b_c"), "a_b_c");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_snake_case_all_uppercase() {
        assert_eq!(rename_all::to_snake_case("ABC"), "a_b_c");
        assert_eq!(rename_all::to_snake_case("XML"), "x_m_l");
        assert_eq!(rename_all::to_snake_case("A"), "a");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_snake_case_with_numbers() {
        assert_eq!(rename_all::to_snake_case("field1Name2"), "field1_name2");
        assert_eq!(rename_all::to_snake_case("v2ApiUrl"), "v2_api_url");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_snake_case_mixed_underscore_uppercase() {
        assert_eq!(rename_all::to_snake_case("_Hello"), "_hello");
        assert_eq!(rename_all::to_snake_case("hello_World"), "hello_world");
        assert_eq!(rename_all::to_snake_case("hello__World"), "hello__world");
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_snake_case_single_char() {
        assert_eq!(rename_all::to_snake_case("a"), "a");
        assert_eq!(rename_all::to_snake_case("Z"), "z");
    }

    // ---- rename_all: roundtrip ----

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn roundtrip_rename_case() {
        let words = &["hello_world", "user_name", "a_b_c", "single", "field1_name2"];
        for w in words {
            let camel = rename_all::to_camel_case(w);
            let back = rename_all::to_snake_case(&camel);
            assert_eq!(&back, w);
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    #[test]
    fn to_camel_then_snake_is_idempotent() {
        let inputs = &["helloWorld", "hello_world", "aBC", "single", "v2ApiUrl"];
        for input in inputs {
            let snake = rename_all::to_snake_case(input);
            let again = rename_all::to_snake_case(&snake);
            assert_eq!(&again, &snake);
        }
    }
}
