use std::str::FromStr;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref LABEL_REGEX: Regex =
        Regex::new(r"iflabel\s*\{\s*(?P<label>[\w\s,]+)?\s*\}").unwrap();
}

#[derive(Debug, PartialEq, Clone, Default)]
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

impl FromStr for Label {
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
            None => Err(()),
        }
    }

    type Err = ();
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
