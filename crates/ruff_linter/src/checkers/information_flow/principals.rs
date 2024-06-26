use lazy_static::lazy_static;
use regex::Regex;
use ruff_python_index::Indexer;
use ruff_source_file::Locator;
use ruff_text_size::TextRange;
use std::str::FromStr;

lazy_static! {
    static ref PRINCIPAL_REGEX: Regex =
        Regex::new(r"ifprincipals\s*\{\s*(?P<principals>[\w\s,]+)\s*\}").unwrap();
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct Principals {
    pub(crate) principals: Vec<String>,
    pub(crate) range: Option<TextRange>,
}

impl Principals {
    #[allow(dead_code)]
    pub(self) fn new(principals: Vec<String>) -> Self {
        Self {
            principals,
            range: None,
        }
    }

    #[allow(dead_code)]
    pub(self) fn new_with_range(principals: Vec<String>, range: TextRange) -> Self {
        Self {
            principals,
            range: Some(range),
        }
    }

    #[allow(dead_code)]
    pub(self) fn new_from_str(principals: Vec<&str>) -> Self {
        Self {
            principals: principals.iter().map(|s| (*s).to_string()).collect(),
            range: None,
        }
    }

    #[allow(dead_code)]
    pub(self) fn new_from_str_with_range(principals: Vec<&str>, range: TextRange) -> Self {
        Self {
            principals: principals.iter().map(|s| (*s).to_string()).collect(),
            range: Some(range),
        }
    }

    pub(crate) fn to_vec(&self) -> Vec<String> {
        self.principals.clone()
    }

    pub(crate) fn new_empty() -> Self {
        Self {
            principals: vec![],
            range: None,
        }
    }

    pub(crate) fn concat(&mut self, other: &Principals) {
        self.principals.extend(other.principals.clone());
    }

    pub(crate) fn add_principle(&mut self, principal: &String) {
        self.principals.push(principal.clone());
    }
}

impl ToString for Principals {
    fn to_string(&self) -> String {
        format!(
            "ifprincipals {{ {} }}",
            self.principals
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl FromStr for Principals {
    /// Parses a string of principals e.g. from a comment
    /// ```python
    /// # ifprincipals { alice, bob }
    /// ```
    ///
    /// into the Principals struct with the principals `['alice', 'bob']`
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match PRINCIPAL_REGEX.captures(string) {
            Some(captures) => {
                let principals = captures
                    .name("principals")
                    .unwrap()
                    .as_str()
                    .replace('\n', "")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>();
                Ok(Principals {
                    principals,
                    range: None,
                })
            }
            None => Err(()),
        }
    }

    type Err = (); // TODO: Provide a concrete type for the Err associated type
}

#[test]
fn test_parse_principals() {
    let principals = "ifprincipals { alice, bob }".parse::<Principals>().unwrap();
    assert_eq!(principals.principals, vec!["alice", "bob"]);

    let principals = "ifprincipals { \n  alice,\n  bob,\n}"
        .parse::<Principals>()
        .unwrap();
    assert_eq!(principals.principals, vec!["alice", "bob"]);
}

/// Initiate the principals list
pub(super) fn initiate_principals(indexer: &Indexer, locator: &Locator) -> Principals {
    let mut principals = Principals::new_empty();
    // TODO: Implement logic to extract principals from block comments with comment_ranges.block_comments()
    for comment_range in indexer.comment_ranges() {
        let comment = locator.slice(comment_range).replace('#', "");
        if let Ok(_principals) = comment.parse::<Principals>() {
            principals.range = Some(comment_range.clone());
            principals.concat(&_principals);
        }
    }

    principals
}

#[cfg(test)]
mod initiate_principals_tests {
    use ruff_text_size::TextSize;

    use super::*;

    #[test]
    fn test_initiate_principals_with_principals() {
        use ruff_python_parser::tokenize;
        use ruff_python_parser::Mode;

        let source: &str = r#"
# ifprincipals { alice, bob }

# This is a comment
x = 1
"#;
        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let principals = initiate_principals(&indexer, &locator);
        assert_eq!(
            principals,
            Principals::new_from_str_with_range(
                vec!["alice", "bob"],
                TextRange::new(TextSize::new(1), TextSize::new(30))
            )
        );
    }

    #[test]
    fn test_initiate_principals_with_principals_not_first_comment() {
        use ruff_python_parser::tokenize;
        use ruff_python_parser::Mode;

        let source: &str = r#"
# This is a comment
# ifprincipals { alice, bob }

x = 1
"#;
        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let principals = initiate_principals(&indexer, &locator);
        assert_eq!(
            principals,
            Principals::new_from_str_with_range(
                vec!["alice", "bob"],
                TextRange::new(TextSize::new(21), TextSize::new(50))
            )
        );
    }

    #[test]
    fn test_initiate_principals_no_principals() {
        use ruff_python_parser::tokenize;
        use ruff_python_parser::Mode;

        let source: &str = r#"
# This is a comment
x = 1
"#;
        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let principals = initiate_principals(&indexer, &locator);
        let empty: Vec<String> = vec![];
        assert_eq!(principals, Principals::new(empty));
    }

    #[test]
    fn test_initiate_principals_multiple_principals_concat() {
        use ruff_python_parser::tokenize;
        use ruff_python_parser::Mode;

        let source: &str = r#"
# ifprincipals { alice, bob }
# ifprincipals { charlie, dean }
# This is a comment
x = 1
"#;
        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let principals = initiate_principals(&indexer, &locator);
        assert_eq!(
            principals,
            Principals::new_from_str_with_range(
                vec!["alice", "bob", "charlie", "dean"],
                TextRange::new(TextSize::new(31), TextSize::new(63))
            )
        );
    }
}
