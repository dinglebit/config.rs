//! Configuration from the environment variables.

use std::env;

use crate::Config;

#[derive(Debug, PartialEq)]
pub struct Environment {
    prefix: String,
}

impl Environment {
    /// Create a new environment configuration which will prefix keys
    /// with the given prefix and an underscore (e.g. prefix = "foo"
    /// => "foo_*"). An empty prefix will not prepend anything.
    ///
    /// Getting a value from the environment will try to make the key
    /// more environment-variable-like . '.' and '/' are replaced with
    /// '_' and everything is upper-cased. If the prefix is 'foo',
    /// then a get for 'my.app.secret' would look for
    /// 'FOO_MY_APP_SECRET'.
    pub fn new(prefix: &str) -> Self {
        let prefix = match prefix.len() > 0 {
            true => prefix.to_owned() + "_",
            false => "".to_string(),
        };
        Self { prefix: prefix }
    }
}

impl Config for Environment {
    /// Get a value from the environment using the given key. '.' and
    /// '/' are replaced with '_' and everything is upper-cased. If the
    /// prefix is 'foo', then a get for 'my.app.secret' would look for
    /// 'FOO_MY_APP_SECRET'.
    fn get(&self, key: &str) -> Option<String> {
        // Make the key more environment variable like.
        let key = self.prefix.to_owned() + key;
        let key = key.replace(".", "_").replace("/", "_");
        let key = key.to_uppercase();

        match env::var(key) {
            Ok(value) => Some(value),
            Err(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::env::Environment;
    use crate::Config;
    use std::env;

    #[test]
    fn new() {
        assert_eq!(
            Environment::new("test"),
            Environment {
                prefix: "test_".to_string()
            }
        );
        assert_eq!(
            Environment::new(""),
            Environment {
                prefix: "".to_string()
            }
        );
    }

    #[test]
    fn get() {
        let e = Environment::new("test_get");
        env::set_var("TEST_GET_FOO_BAR", "baz");
        assert_eq!(e.get("foo.bar"), Some("baz".to_string()));
        assert_eq!(e.get("foo/bar"), Some("baz".to_string()));
        env::remove_var("TEST_GET_FOO_BAR");
        assert_eq!(e.get("foo.bar"), None);
    }
}
