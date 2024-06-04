use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use rustc_hash::FxHashSet;
use std::{
    ops::{Add, AddAssign},
    str::FromStr,
};

lazy_static! {
    static ref LABEL_REGEX: Regex =
        Regex::new(r"iflabel\s*\{\s*(?P<label>[\w\s,]+)?\s*\}").unwrap();
    static ref FUNCTION_LABEL_REGEX: Regex =
        Regex::new(r"iflabel\s+fn\s*\(\s*(?P<args>(?:[a-zA-Z]+\s*:\s*\{[[a-zA-Z], ]*\}\s*,\s*)*[a-zA-Z]+\s*:\s*\{[[a-zA-Z](,\s)*\}]*\})?\s*\)\s*(\{\s*(?P<returnlabel>([a-zA-Z](,\s*)?)+)?\s*\})?").unwrap();
    static ref ARG_LABEL_REGEX: Regex = Regex::new(r"\s*(?P<argname>[\w]+)\s*:\s*\{(?P<label>[\w\s*,]+)?\}").unwrap();
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub(crate) struct Label {
    pub(crate) principals: FxHashSet<String>,
}

impl Label {
    pub(crate) fn new(principals: Vec<String>) -> Self {
        Self {
            principals: FxHashSet::from_iter(principals.into_iter()),
        }
    }

    pub(crate) fn is_public(&self) -> bool {
        self.principals.is_empty()
    }

    #[allow(dead_code)]
    pub(crate) fn new_public() -> Self {
        Self {
            principals: FxHashSet::default(),
        }
    }

    pub(crate) fn to_string(&self) -> String {
        if self.is_public() {
            return "{}".to_string();
        }
        let principals = self.principals.iter().join(", ");
        format!("{{{}}}", principals)
    }

    /// E.g true for AB.is_higher_in_lattice_path(A) and AB.is_higher_in_lattice_path(B),
    /// but false for A.is_higher_in_lattice_path(B)
    ///
    /// ```latex
    ///       AB
    ///      / \
    ///     A   B
    ///      \ /
    ///       0
    /// ```
    fn is_higher_in_lattice_path(&self, label: &Label) -> bool {
        // If self has more principals, then it is higher in the lattice
        if self.principals.len() > label.principals.len() {
            // Check if the label is a subset of the self
            for principal in &label.principals {
                if !self.principals.contains(principal) {
                    return false;
                }
            }

            return true;
        }

        // If the labels have the same principals, then you can "convert" between them
        return false;
    }
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self == other {
            return Some(std::cmp::Ordering::Equal);
        }

        if self.lt(other) {
            return Some(std::cmp::Ordering::Less);
        }

        if self.gt(other) {
            return Some(std::cmp::Ordering::Greater);
        }

        None
    }

    fn lt(&self, other: &Self) -> bool {
        other.is_higher_in_lattice_path(self)
    }

    fn le(&self, other: &Self) -> bool {
        self == other || self.lt(other)
    }

    fn gt(&self, other: &Self) -> bool {
        self.is_higher_in_lattice_path(other)
    }

    fn ge(&self, other: &Self) -> bool {
        self == other || self.gt(other)
    }
}

impl Add for Label {
    type Output = Label;

    fn add(self, other: Label) -> Self::Output {
        let mut principals = self.principals.clone();
        principals.extend(other.principals.clone());
        Label { principals }
    }
}

impl AddAssign for Label {
    fn add_assign(&mut self, other: Label) {
        self.principals.extend(other.principals)
    }
}

impl<'a> Add<&'a Label> for Label {
    type Output = Label;

    fn add(mut self, other: &'a Label) -> Self::Output {
        self.principals.extend(other.principals.clone());
        self
    }
}

impl<'a> AddAssign<&'a Label> for Label {
    fn add_assign(&mut self, other: &'a Label) {
        self.principals.extend(other.principals.clone())
    }
}

#[test]
fn add_labels() {
    let a = &Label::new(vec!["alice".to_string()]);
    let b = &Label::new(vec!["bob".to_string()]);
    let ab = &Label::new(vec!["alice".to_string(), "bob".to_string()]);
    let p = &Label::new_public();

    assert_eq!(a.clone() + b, ab.clone());
    assert_eq!(a.clone() + p, a.clone());
    assert_eq!(p.clone() + a, a.clone());

    assert_eq!(a.clone() + a, a.clone());
    assert_eq!(b.clone() + b, b.clone());
    assert_eq!(p.clone() + p, p.clone());

    assert_eq!(ab.clone() + a, ab.clone());
    assert_eq!(ab.clone() + b, ab.clone());
    assert_eq!(ab.clone() + p, ab.clone());
}

#[test]
fn test_label_ordering() {
    let a: Label = Label::new(vec!["alice".to_string()]);
    let b: Label = Label::new(vec!["bob".to_string()]);
    let ab: Label = Label::new(vec!["alice".to_string(), "bob".to_string()]);
    let p: Label = Label::new_public();

    assert!(a == a);
    assert!(b == b);
    assert!(ab == ab);
    assert!(p == p);

    assert!(a < ab);
    assert!(b < ab);
    assert!(p < a);
    assert!(p < b);
    assert!(p < ab);

    assert!(a <= a);
    assert!(a <= ab);
    assert!(b <= ab);
    assert!(p <= a);
    assert!(!(b <= a));
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub(crate) struct FunctionLabel {
    pub(crate) argument_labels: Vec<(String, Label)>,
    pub(crate) return_label: Label,
}
impl FunctionLabel {
    #[allow(dead_code)]
    pub(crate) fn to_string(&self) -> String {
        let argument_labels = self
            .argument_labels
            .iter()
            .map(|(name, label)| format!("{}: {}", name, label.to_string()))
            .collect::<Vec<String>>()
            .join(", ");

        format!(
            "{{ {} }} {{{}}}",
            argument_labels,
            self.return_label.to_string()
        )
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
                    let principals = FxHashSet::from_iter(
                        label
                            .as_str()
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty()),
                    );
                    Ok(Label { principals })
                }
                None => Ok(Label::new_public()),
            },
            None => Err(LabelParseError),
        }
    }
}

impl FromStr for FunctionLabel {
    type Err = LabelParseError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match FUNCTION_LABEL_REGEX.captures(string) {
            Some(captures) => {
                let argument_labels = match captures.name("args") {
                    Some(args) => {
                        let mut labels = vec![];
                        ARG_LABEL_REGEX
                            .captures_iter(args.into())
                            .for_each(|capture| {
                                if let Some(argname) = capture.name("argname") {
                                    if let Some(labels_string) = capture.name("label") {
                                        let label = Label::new(
                                            labels_string
                                                .as_str()
                                                .split(',')
                                                .map(str::trim)
                                                .map(str::to_string)
                                                .collect(),
                                        );
                                        labels.push((argname.as_str().to_string(), label));
                                    } else {
                                        labels.push((
                                            argname.as_str().to_string(),
                                            Label::new_public(),
                                        ));
                                    }
                                }
                            });

                        labels
                    }
                    None => vec![],
                };
                let return_label = match captures.name("returnlabel") {
                    Some(label) => {
                        let label = label.as_str();
                        Label::new(label.split(',').map(|s| s.trim().to_string()).collect())
                    }
                    None => Label::new_public(),
                };
                Ok(FunctionLabel {
                    argument_labels,
                    return_label,
                })
            }
            None => Err(LabelParseError),
        }
    }
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
        let principals = ["alice", "bob", "charlie"];

        let mut label_string = String::from("iflabel {");

        let mut i = 0;
        while i < principals.len() {
            let formatted_string = format!("{}, {}", label_string, principals[i]);
            label_string = formatted_string;

            let label = (format!("{label_string}}}")).parse::<Label>().unwrap();
            assert_eq!(
                label,
                Label::new(principals[..=i].iter().map(|s| (*s).to_string()).collect())
            );
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

    #[test]
    fn test_label_conversion() {
        let label1 = Label::new(vec!["alice".to_string(), "bob".to_string()]);
        let label2 = Label::new(vec!["alice".to_string()]);
        let label3 = Label::new(vec!["bob".to_string()]);

        assert!(label1.is_higher_in_lattice_path(&label2));
        assert!(label1.is_higher_in_lattice_path(&label3));

        assert!(!label2.is_higher_in_lattice_path(&label1));
        assert!(!label3.is_higher_in_lattice_path(&label1));
        assert!(!label2.is_higher_in_lattice_path(&label3));
    }
}
