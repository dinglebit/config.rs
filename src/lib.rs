//! Simplified configuration management.
//!
//! This configuration package isn't meant to solve all of the
//! configuration needs you'll ever need. Instead, it provides a trait
//! for a config and then allows for including other confugiration
//! systems as needed.
//!
//! A simple environment config and file config are provided.
//!
//! ```
//! use dinglebit_config::{Config, Environment, MultiConfig, Simple};
//! use std::collections::HashMap;
//!
//! fn main() {
//!     let mut m = HashMap::new();
//!     m.insert("foo", "bar");
//!     let cfg = MultiConfig::new(vec![
//!         Box::new(m),
//!         Box::new(Simple::from_str("baz=foo").unwrap()),
//!     ]);
//!
//!     assert_eq!(cfg.must_get("foo"), "bar".to_string());
//!     assert_eq!(cfg.must_get("baz"), "foo".to_string());
//!     assert!(cfg.get("bar").is_none());
//! }

use std::collections::HashMap;

pub mod env;
pub mod multi;
pub mod simple;

pub use env::Environment;
pub use multi::MultiConfig;
pub use simple::{Error, Simple};

/// The main trait for this package. This should be implemented if you
/// want to use this package with your configuration systems.
pub trait Config {
    /// Returns the value associated with the given key.
    fn get(&self, key: &str) -> Option<String>;

    /// Similar to `get` but panics if there is no value.
    fn must_get(&self, key: &str) -> String {
        self.get(key).unwrap()
    }

    /// Get the value as a string or panics if one isn't found.
    fn string(&self, key: &str) -> String {
        self.get(key).unwrap()
    }

    /// Get the value as an integer or panics if one isn't found or
    /// cannot be parsed.
    fn int(&self, key: &str) -> i64 {
        self.must_get(key).parse::<i64>().unwrap()
    }

    /// Get the value as a float or panics if one isn't found or
    /// cannot be parsed.
    fn float(&self, key: &str) -> f64 {
        self.must_get(key).parse::<f64>().unwrap()
    }

    /// Get the value as a bool or panics if one isn't found or cannot
    /// be parsed. The following case-insensitive values are considered
    /// true: t, true, 1, y, yes. All other values are considered
    /// false.
    fn bool(&self, key: &str) -> bool {
        match self.must_get(key).to_lowercase().as_str() {
            "t" => true,
            "true" => true,
            "1" => true,
            "y" => true,
            "yes" => true,
            _ => false,
        }
    }

    /// Get the value as a duration or panics if one isn't found or
    /// can't be parsed. Thre doesn't appear to be a parsing function
    /// for a duration, so it attempts to convert to an integer and use
    /// that as the number of seconds.
    fn duration(&self, key: &str) -> chrono::Duration {
        // There doesn't seem to be a parse function for
        // chrono::Duration. We just assume i64 seconds.
        chrono::Duration::seconds(self.int(key))
    }

    /// Get the value as a duration or panics if one isn't found or it
    /// can't be parsed. It uses RFC339 to parse it.
    fn datetime(&self, key: &str) -> chrono::DateTime<chrono::Utc> {
        chrono::DateTime::<chrono::Utc>::from_utc(
            chrono::DateTime::parse_from_rfc3339(self.must_get(key).as_str())
                .unwrap()
                .naive_utc(),
            chrono::Utc,
        )
    }

    /// Get a list or panics if one isn't found. The list should be a
    /// comma-delimited list surrouned by brackets (e.g. [1, 2, 3] =>
    /// vec!["1", "2", "3"].
    fn list(&self, key: &str) -> Vec<String> {
        let s = self.must_get(key);
        let s = s.trim_matches(|c| c == '[' || c == ']' || char::is_whitespace(c));
        s.split(',')
            .map(|p| p.trim().to_string())
            .collect::<Vec<String>>()
    }

    /// Get a map or panics if one isn't found. The list should be a
    /// comma-delimited list surrouned by braces with key/value pairs
    /// associated with => (e.g. {a=>1, b=>2, c=>3} => ((a,1), (b,2),
    /// (c,3))).
    fn map(&self, key: &str) -> HashMap<String, String> {
        let s = self.must_get(key);
        let s = s.trim_matches(|c| c == '{' || c == '}' || char::is_whitespace(c));
        s.split(',')
            .map(|p| {
                let parts = p.split("=>").map(|k| k.trim()).collect::<Vec<&str>>();
                if parts.len() < 2 {
                    (parts[0], "")
                } else {
                    (parts[0], parts[1])
                }
            })
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<String, String>>()
    }
}

/// Create a config from a list of key/value pairs.
#[macro_export]
macro_rules! default_config(
    { $($key:expr => $value:expr),+ } => {
        {
            let mut m: ::std::collections::HashMap<&str, &str> = ::std::collections::HashMap::new();
            $(
                m.insert($key, $value);
            )+
            m
        }
     };
);

impl Config for HashMap<&str, &str> {
    fn get(&self, key: &str) -> Option<String> {
        match self.get(key) {
            None => None,
            Some(v) => Some(v.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use chrono::{TimeZone, Utc};
    use lazy_static::lazy_static;
    use std::collections::HashMap;

    #[test]
    fn default() {
        let config = default_config! {
            "foo" => "bar",
            "bar" => "baz",
            "baz" => "foo"
        };
        assert_eq!(config.string("foo"), "bar".to_string());
    }

    #[test]
    fn hash_map() {
        use std::collections::HashMap;
        let mut m = HashMap::new();
        m.insert("foo", "bar");
        assert_eq!(m.must_get("foo"), "bar".to_string());
        assert!(m.get("bar").is_none());
    }

    lazy_static! {
        static ref HASHMAP: HashMap<&'static str, &'static str> = {
            let mut m = HashMap::new();
            m.insert("foo", "bar");
            m.insert("int", "100");
            m.insert("float", "-2.4");
            m.insert("bool", "t");
            m.insert("duration", "50");
            m.insert("datetime", "2015-05-15T05:05:05+00:00");
            m.insert("list", "[1, 2, 3]");
            m.insert("map", "{a=>1, b=>2, c=>3}");
            m
        };
    }

    macro_rules! test_gets {
        ($(($name:ident, $test:expr): $exp:expr,)*) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(
                        $test,
                        $exp
                    );
                }

            )*

        }
    }

    test_gets! {
        (string, HASHMAP.string("foo")): "bar".to_string(),
        (int, HASHMAP.int("int")): 100,
        (float, HASHMAP.float("float")): -2.4,
        (bool, HASHMAP.bool("bool")): true,
        (duration, HASHMAP.duration("duration")): chrono::Duration::seconds(50),
        (datetime, HASHMAP.datetime("datetime")): Utc.ymd(2015, 5, 15).and_hms(5, 5, 5),
        (list, HASHMAP.list("list")): vec!["1", "2", "3"],
        (map, HASHMAP.map("map")): {
            let mut m: HashMap<String, String> = HashMap::new();
            m.insert("a".to_string(), "1".to_string());
            m.insert("b".to_string(), "2".to_string());
            m.insert("c".to_string(), "3".to_string());
            m
        },
    }
}
