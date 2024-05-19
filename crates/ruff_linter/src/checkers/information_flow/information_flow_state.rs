use rustc_hash::FxHashMap;
use std::collections::VecDeque;

use ruff_python_index::Indexer;
use ruff_python_semantic::BindingId;
use ruff_python_trivia::CommentRanges;
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};

use super::{
    label::Label,
    principals::{initiate_principals, Principals},
};

/// State of the information flow
#[derive()]
pub(crate) struct InformationFlowState {
    // The current principles of the program, e.g. ['alice', 'bob']
    #[allow(dead_code)]
    principals: Principals,
    // The current scope level queue. The level is updated according to the scope by popping and
    pc: VecDeque<String>,
    // Map from variable name to
    variable_map: FxHashMap<BindingId, Label>,
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
    pub(crate) fn pc(&self) -> String {
        return match self.pc.front() {
            Some(pc) => pc.clone(),
            None => String::new(),
        };
    }

    pub(crate) fn variable_map(&self) -> &FxHashMap<BindingId, Label> {
        &self.variable_map
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
        // Add to variable_map
        if let Some((label, ..)) = read_variable_label_from_source(range, locator, comment_ranges) {
            self.variable_map.insert(binding_id, label);
        } else {
            // No label comment, add public label
            self.variable_map.insert(binding_id, Label::new_public());
        }
    }

    pub(crate) fn principals(&self) -> &Principals {
        &self.principals
    }
}

/// Read the variable label from the source code and return [`Label`] if found
/// and the [`TextRange`] of the label
pub(crate) fn read_variable_label_from_source(
    range: TextRange,
    locator: &Locator,
    comment_ranges: &CommentRanges,
) -> Option<(Label, TextRange)> {
    // Find comment on same line
    let line_range = locator.line_range(range.start());
    let inline_label_comment = comment_ranges.comments_in_range(line_range).first();

    if let Some(comment_range) = inline_label_comment {
        let comment_text: &str = &locator.slice(comment_range).replace('#', "");
        if let Ok(label) = comment_text.parse::<Label>() {
            return Some((label, comment_range.clone()));
        }
    } else {
        // Find comment on previous line if it exists
        let start_range = range.start().to_u32();
        let preline_label_comment = if start_range != 0 {
            comment_ranges
                .comments_in_range(locator.line_range(TextSize::from(start_range - 1))) // Previous line
                .first()
        } else {
            return None;
        };

        if let Some(comment_range) = preline_label_comment {
            let comment_text: &str = &locator.slice(comment_range).replace('#', "");
            if let Ok(label) = comment_text.parse::<Label>() {
                return Some((label, comment_range.clone()));
            } else {
                return None;
            }
        }
    }
    return None;
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
                    if let Stmt::Assign(StmtAssign {
                        targets: _,
                        value: _,
                        range,
                    }) = stmt
                    {
                        let binding_id: BindingId = id;
                        id = (id.as_u32() + 1).into();
                        state.add_variable_label_binding(
                            binding_id,
                            range,
                            &locator,
                            comment_ranges,
                        );
                    }
                }
            }
            Err(_) => panic!("Failed to parse module"),
        }

        assert!(state.variable_map.contains_key(&BindingId::from(0u32)));
        assert!(state.variable_map.contains_key(&BindingId::from(1u32)));
        assert!(state.variable_map.contains_key(&BindingId::from(2u32)));

        let label1 = &state.variable_map[&BindingId::from(0u32)];
        let label2 = &state.variable_map[&BindingId::from(1u32)];
        let label3 = &state.variable_map[&BindingId::from(2u32)];

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
                            let binding_id: BindingId = id;
                            id = (id.as_u32() + 1).into();
                            state.add_variable_label_binding(
                                binding_id,
                                range,
                                &locator,
                                comment_ranges,
                            );
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => panic!("Failed to parse module"),
        }

        assert!(state.variable_map.len() != 0);
        assert!(state.variable_map.contains_key(&BindingId::from(0u32)));
        assert!(state.variable_map.get(&BindingId::from(0u32)).unwrap() == &Label::new_public());
    }

    #[test]
    fn test_information_flow_state_add_public_label_to_variable_map() {
        let source: &str = r#"a = 1
b = 2 # iflabel {}
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
                    if let Stmt::Assign(StmtAssign {
                        targets: _,
                        value: _,
                        range,
                    }) = stmt
                    {
                        let binding_id: BindingId = id;
                        id = (id.as_u32() + 1).into();
                        state.add_variable_label_binding(
                            binding_id,
                            range,
                            &locator,
                            comment_ranges,
                        );
                    }
                }
            }
            Err(_) => panic!("Failed to parse module"),
        }

        assert!(state.variable_map.contains_key(&BindingId::from(0u32)));
        assert!(state.variable_map.contains_key(&BindingId::from(1u32)));
        assert_eq!(
            &state.variable_map[&BindingId::from(0u32)],
            &Label::new_public()
        );
        assert_eq!(
            &state.variable_map[&BindingId::from(1u32)],
            &Label::new_public()
        );
    }
}
