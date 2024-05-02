use lazy_static::lazy_static;
use regex::Regex;
use std::str::FromStr;

lazy_static! {
    static ref LABEL_REGEX: Regex =
        Regex::new(r"iflabel\s*\{\s*(?P<label>[\w\s,]+)?\s*\}").unwrap();
}

#[derive(Debug, PartialEq, Clone, Default, Eq)]
pub(crate) struct Label {
    pub(crate) principals: Vec<String>,
}
impl Label {
    #[allow(dead_code)]
    pub(crate) fn new(principals: Vec<String>) -> Self {
        Self { principals }
    }

    pub(crate) fn is_public(&self) -> bool {
        self.principals.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn new_public() -> Self {
        Self { principals: vec![] }
    }
}
#[derive(Debug, Clone)]
pub(crate) struct LabelParseError;

impl std::fmt::Display for LabelParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Label parse error")
    }
}

impl FromStr for Label {
    type Err = LabelParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match LABEL_REGEX.captures(string) {
            Some(captures) => match captures.name("label") {
                Some(label) => {
                    let principals = label
                        .as_str()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect::<Vec<String>>();
                    return Ok(Label { principals });
                }
                None => return Ok(Label::new_public()),
            },
            None => Err(LabelParseError),
        }
    }
}

/// Check labels direction convertion i.e. you can move down in the lattice, not up
pub(crate) fn can_convert_label(from_label: &Label, to_label: &Label) -> bool {
    // If the test label is public, then it is never more restrictive
    if to_label.is_public() {
        return true;
    }

    // If the to_label has more principals, then it is more restrictive
    if from_label.principals.len() > to_label.principals.len() {
        // Check if the to_label is a subset of the from_label
        for principal in &to_label.principals {
            if !from_label.principals.contains(principal) {
                return false;
            }
        }
        return true;
    }

    // If the conv_label has the same principals, then it is not more restrictive
    return to_label.principals.len() == from_label.principals.len()
        && to_label.principals != from_label.principals;
}

#[cfg(test)]
mod test_labels {
    use super::Label;
    #[test]
    fn test_label_parse_one() {
        let label = "iflabel { alice }".parse::<Label>().unwrap();
        assert_eq!(label, Label::new(vec!["alice".to_string()]));
    }

    #[test]
    fn test_label_parse_multiple() {
        let principals = vec!["alice", "bob", "charlie"];

        let mut label_string = String::from("iflabel {");

        let mut i = 0;
        while i < principals.len() {
            let formatted_string = format!("{}, {}", label_string, principals[i]);
            label_string = formatted_string;

            let label = (format!("{}}}", label_string)).parse::<Label>().unwrap();
            assert_eq!(label, Label::new(principals[..i+1].iter().map(|s| s.to_string()).collect()));
            i += 1;
        }
    }

    #[test]
    fn test_label_parse_public() {
        let label = "iflabel {}".parse::<Label>().unwrap();
        assert_eq!(label, Label::new_public());

        let label = "iflabel { }".parse::<Label>().unwrap();
        assert_eq!(label, Label::new_public());
    }

    #[test]
    fn test_failing_regex_should_throw() {
        assert!("".parse::<Label>().is_err());
    }
}
