use std::{collections::VecDeque, str::FromStr, vec};

use lazy_static::lazy_static;
use regex::Regex;
use ruff_python_index::Indexer;
use ruff_python_semantic::BindingId;
use ruff_python_trivia::CommentRanges;
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};
use rustc_hash::FxHashMap;

lazy_static! {
    static ref PRINCIPAL_REGEX: Regex =
        Regex::new(r"ifprincipals\s*\{\s*(?P<principals>[\w\s,]+)\s*\}").unwrap();
    static ref LABEL_REGEX: Regex = Regex::new(r"iflabel\s*\{\s*(?P<label>[\w\s,]+)\s*\}").unwrap();
}

/// State of the information flow
#[derive()]
pub(crate) struct InformationFlowState {
    // The current principles of the program, e.g. ['alice', 'bob']
    principals: Principals,
    // The current scope level queue. The level is updated according to the scope by popping and
    pc: VecDeque<String>,
    // Map from variable name to
    variable_map: FxHashMap<BindingId, Label>,
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct Label {
    principals: Vec<String>,
}

impl Label {
    pub(crate) fn new(principals: Vec<String>) -> Self {
        Self { principals }
    }

    pub(crate) fn is_public(&self) -> bool {
        self.principals.is_empty()
    }

    pub(crate) fn new_public() -> Self {
        Self { principals: vec![] }
    }
}

impl FromStr for Label {
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match LABEL_REGEX.captures(string) {
            Some(captures) => {
                let principals = captures
                    .name("label")
                    .unwrap()
                    .as_str()
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect::<Vec<String>>();
                Ok(Label { principals })
            }
            None => Err(()),
        }
    }

    type Err = ();
}

impl InformationFlowState {
    pub(crate) fn new(indexer: &Indexer, locator: &Locator) -> Self {
        Self {
            principals: initiate_principals(indexer, locator),
            variable_map: FxHashMap::default(),
            pc: VecDeque::default(),
        }
    }

    /// Return the current level of the information flow state
    pub(crate) fn get_pc(&self) -> String {
        return match self.pc.front() {
            Some(pc) => pc.clone(),
            None => "".to_string(),
        };
    }

    pub(crate) fn get_label(&self, binding_id: BindingId) -> Option<Label> {
        return self.variable_map.get(&binding_id).cloned();
    }

    pub(crate) fn add_variable_label_binding(
        &mut self,
        binding_id: BindingId,
        range: TextRange,
        locator: &Locator,
        comment_ranges: &CommentRanges,
    ) {
        // Find comment on same line
        // Regex label
        // Add to variable_map
        let line_range = locator.line_range(range.start());
        let label_comment = comment_ranges.comments_in_range(line_range).first();
        match label_comment {
            Some(comment) => {
                let comment_text: &str = &locator.slice(comment).replace("#", "");
                if let Ok(label) = comment_text.parse::<Label>() {
                    self.variable_map.insert(binding_id, label);
                }
            }
            // No comment on same line, check previous line
            None => {
                let previous_line_range =
                    locator.line_range(TextSize::from(range.start().to_u32() - 1));
                let label_comment = comment_ranges
                    .comments_in_range(previous_line_range)
                    .first();
                match label_comment {
                    Some(comment) => {
                        let comment_text: &str = &locator.slice(comment).replace("#", "");
                        if let Ok(label) = comment_text.parse::<Label>() {
                            self.variable_map.insert(binding_id, label);
                        }
                    }
                    None => {}
                }
            }
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

#[derive(Debug, PartialEq)]
struct Principals {
    principals: Vec<String>,
}

impl Principals {
    fn new(principals: Vec<String>) -> Self {
        Self { principals }
    }

    fn new_from_str(principals: Vec<&str>) -> Self {
        Self {
            principals: principals.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn new_empty() -> Self {
        Self { principals: vec![] }
    }

    fn concat(&mut self, other: &Principals) {
        self.principals.extend(other.principals.clone());
    }
}

impl FromStr for Principals {
    /// Parses a string of principals e.g. from a comment
    /// ```
    /// ifprincipals {
    ///   alice,
    ///   bob
    /// }
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
                Ok(Principals { principals })
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
fn initiate_principals(indexer: &Indexer, locator: &Locator) -> Principals {
    let mut principals = Principals::new_empty();
    // TODO: Implement logic to extract principals from block comments with comment_ranges.block_comments()
    for range in indexer.comment_ranges() {
        let comment = locator.slice(range).replace("#", "");
        if let Ok(_principals) = comment.parse::<Principals>() {
            principals.concat(&_principals);
        }
    }

    return principals;
}

#[cfg(test)]
mod initiate_principals_tests {
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
        assert_eq!(principals, Principals::new_from_str(vec!["alice", "bob"]));
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
        assert_eq!(principals, Principals::new_from_str(vec!["alice", "bob"]));
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
            Principals::new_from_str(vec!["alice", "bob", "charlie", "dean"])
        );
    }
}

#[cfg(test)]
mod information_flow_state_tests {
    use ruff_python_ast::{Stmt, StmtAssign};
    use ruff_python_parser::{parse_program, tokenize, Mode};

    use super::*;

    #[test]
    fn test_information_flow_state_add_assign_label_to_variable_map() {
        let source: &str = r#"
a = 1 # iflabel {alice}
b = 2 # iflabel {bob, alice}

# iflabel {alice}
c = 3
"#;
        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let comment_ranges = indexer.comment_ranges();
        let result = parse_program(source);
        let mut state = InformationFlowState::new(&indexer, &locator);

        let mut id: BindingId = BindingId::from(0u32);

        match result {
            Ok(module) => {
                let stmts = module.body;
                for stmt in stmts {
                    match stmt {
                        Stmt::Assign(StmtAssign {
                            targets: _,
                            value: _,
                            range,
                        }) => {
                            let binding_id: BindingId = id.clone();
                            id = (id.as_u32() + 1).into();
                            state.add_variable_label_binding(
                                binding_id,
                                range,
                                &locator,
                                &comment_ranges,
                            );
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => panic!("Failed to parse module"),
        }

        assert!(state.variable_map.contains_key(&BindingId::from(0u32)));
        assert!(state.variable_map.contains_key(&BindingId::from(1u32)));
        assert!(state.variable_map.contains_key(&BindingId::from(2u32)));

        let label1 = state.variable_map.get(&BindingId::from(0u32)).unwrap();
        let label2 = state.variable_map.get(&BindingId::from(1u32)).unwrap();
        let label3 = state.variable_map.get(&BindingId::from(2u32)).unwrap();

        assert_eq!(
            label1,
            &Label {
                principals: vec!["alice".to_string()]
            }
        );
        assert_eq!(
            label2,
            &Label {
                principals: vec!["bob".to_string(), "alice".to_string()]
            }
        );
        assert_eq!(
            label3,
            &Label {
                principals: vec!["alice".to_string()]
            }
        );
    }

    #[test]
    fn test_information_flow_state_skip_comments_two_lines_above() {
        let source: &str = r#"
# iflabel {alice}

a = 1
"#;
        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let comment_ranges = indexer.comment_ranges();
        let result = parse_program(source);
        let mut state = InformationFlowState::new(&indexer, &locator);

        let mut id: BindingId = BindingId::from(0u32);

        match result {
            Ok(module) => {
                let stmts = module.body;
                for stmt in stmts {
                    match stmt {
                        Stmt::Assign(StmtAssign {
                            targets: _,
                            value: _,
                            range,
                        }) => {
                            let binding_id: BindingId = id.clone();
                            id = (id.as_u32() + 1).into();
                            state.add_variable_label_binding(
                                binding_id,
                                range,
                                &locator,
                                &comment_ranges,
                            );
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => panic!("Failed to parse module"),
        }

        assert!(state.variable_map.len() == 0);
    }
}
