//! Extremely simplistic configuration from a file or string.

use std::collections::HashMap;
use std::fs::read_to_string;

use crate::Config;

#[derive(Debug, PartialEq)]
pub struct Simple {
    values: HashMap<String, String>,
}

#[derive(Debug, PartialEq)]
pub enum Error {
    File(String),
    InvalidKeyValuePair,
}

fn parse_line(line: &str) -> Result<Option<(String, String)>, Error> {
    // Cleanup and check for comments
    let line = line.trim();
    if line.starts_with("#") {
        return Ok(None);
    } else if line.len() < 1 {
        return Ok(None);
    }

    // Split by the equal sign. Expect exactly two.
    let parts: Vec<&str> = line.splitn(2, "=").collect();
    if parts.len() < 2 {
        return Err(Error::InvalidKeyValuePair);
    }

    Ok(Some((
        parts[0].trim().to_string(),
        parts[1].trim().to_string(),
    )))
}

fn parse(s: &str) -> Result<HashMap<String, String>, Error> {
    let mut values = HashMap::new();

    for line in s.split("\n") {
        match parse_line(&line) {
            Err(e) => return Err(e),
            Ok(v) => match v {
                None => continue,
                Some(s) => {
                    values.insert(s.0, s.1);
                }
            },
        }
    }

    Ok(values)
}

impl Simple {
    /// Create a new configuration from the given string. This is an
    /// extremely simple configuration format. It expects key/value
    /// pairs separated by an equal sign. Whitespace is trimmed from
    /// the line as well as each key/value. Lines that begin with `#`
    /// are considered a comment and empty lines are ignored. Thre is
    /// no hierarchy or anything. If you want to provide some
    /// yourself, you can use dot-notation. For example:
    ///
    /// ```
    /// ## i am a comment
    /// mongo.uri = mongodb://localhost/
    /// mongo.db  = test
    /// ```
    pub fn from_str(s: &str) -> Result<Self, Error> {
        Ok(Self { values: parse(s)? })
    }

    /// Similar to `from_str` except that the given path is used as
    /// the contents for the string to parse.
    pub fn from_file(path: &str) -> Result<Self, Error> {
        let file = match read_to_string(path) {
            Ok(s) => s,
            Err(e) => return Err(Error::File(e.to_string())),
        };
        Ok(Self {
            values: parse(&file)?,
        })
    }
}

impl Config for Simple {
    fn get(&self, key: &str) -> Option<String> {
        match self.values.get(key) {
            Some(value) => Some(value.to_string()),
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::simple::{parse_line, Error, Simple};
    use crate::Config;

    use std::array::IntoIter;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[test]
    fn test_parse_line() {
        let tests =
            HashMap::<&str, Result<Option<(String, String)>, Error>>::from_iter(IntoIter::new([
                ("     # comment   ", Ok(None)),
                ("  test", Err(Error::InvalidKeyValuePair)),
                (
                    "  foo    =    bar    ",
                    Ok(Some(("foo".to_string(), "bar".to_string()))),
                ),
            ]));
        tests.iter().for_each(|(k, v)| {
            assert_eq!(parse_line(k), *v);
        });
    }

    #[test]
    fn test_file() {
        // not found
        let exp: Result<Simple, Error> = Err(Error::File(
            "No such file or directory (os error 2)".to_string(),
        ));
        assert_eq!(Simple::from_file("/i/hope/i/do/not/exist.cfg"), exp);

        // our example config
        let cfg = match Simple::from_file("example.cfg") {
            Err(e) => panic!("reading 'example.cfg': {:?}", e),
            Ok(cfg) => cfg,
        };
        assert_eq!(cfg.get("foo"), Some("bar".to_string()));
        assert_eq!(cfg.get("list"), Some("one, two, three".to_string()));
    }
}
