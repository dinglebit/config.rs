//! Combine multiple configs to get configuration values from various
//! places.

use crate::Config;

pub struct MultiConfig {
    configs: Vec<Box<dyn Config>>,
}

impl MultiConfig {
    //! Create a configuration that uses the given list of configs to
    //! try and get values. If a value isn't found, the next config on
    //! the list is consulted. This allows your to create a set of
    //! configs that can override values as needed. For example,
    //! creating a `MultiConfig` with `!vec[environment,
    //! instance-config-file, global-config-file, default-values]`
    //! would provide something like you'd expect in a 12-factor app.
    pub fn new(configs: Vec<Box<dyn Config>>) -> Self {
        Self { configs }
    }
}

impl Config for MultiConfig {
    fn get(&self, key: &str) -> Option<String> {
        for config in self.configs.iter() {
            match config.get(key) {
                Some(value) => return Some(value),
                None => continue,
            };
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::{multi::MultiConfig, Config};

    #[test]
    fn multi() {
        // Create two maps
        use std::collections::HashMap;
        let mut m1 = HashMap::new();
        m1.insert("foo", "bar");
        m1.insert("bar", "baz");
        let mut m2 = HashMap::new();
        m2.insert("foo", "buz");
        m2.insert("buz", "foo");

        let mc = MultiConfig::new(vec![Box::new(m2), Box::new(m1)]);

        assert_eq!(mc.get("foo"), Some("buz".to_string()));
        assert_eq!(mc.get("bar"), Some("baz".to_string()));
        assert_eq!(mc.get("buz"), Some("foo".to_string()));
    }
}
