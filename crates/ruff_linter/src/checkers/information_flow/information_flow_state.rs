use ruff_python_ast::{AnyParameterRef, Expr, Identifier};
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

use ruff_python_index::Indexer;
use ruff_python_semantic::{BindingId, BindingKind};
use ruff_python_trivia::CommentRanges;
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};

use super::{
    label::{FunctionLabel, Label},
    principals::{initiate_principals, Principals},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct PC {
    /// The current PC label
    label: Label,
    /// The range of the statement where the PC is set
    range: TextRange,
}

/// State of the information flow
#[derive()]
pub(crate) struct InformationFlowState {
    // The current principles of the program, e.g. ['alice', 'bob']
    #[allow(dead_code)]
    principals: Principals,
    // The current scope level queue. The level is updated according to the scope by popping and
    pc: VecDeque<PC>,
    // Map from variable name to
    variable_map: FxHashMap<BindingId, Label>,
    // Map from function name to parameter label
    function_parameter_map: FxHashMap<BindingId, FxHashMap<String, Label>>,
}

impl InformationFlowState {
    pub(crate) fn new(indexer: &Indexer, locator: &Locator) -> Self {
        Self {
            principals: initiate_principals(indexer, locator),
            variable_map: FxHashMap::default(),
            pc: VecDeque::default(),
            function_parameter_map: FxHashMap::default(),
        }
    }

    /// Return the current level of the information flow state
    pub(crate) fn get_pc_label(&self) -> Label {
        return match self.pc.front() {
            Some(pc) => pc.label.clone(),
            None => Label::new_public(), // TODO: Should this be public by default?
        };
    }

    pub(crate) fn get_pc_expr_range(&self) -> TextRange {
        return match self.pc.front() {
            Some(pc) => pc.range.clone(),
            None => TextRange::default(),
        };
    }

    /// Set the current level of the information flow state.
    /// If PC is higher from before, add that instead.
    pub(crate) fn push_pc(&mut self, pc: Label, range: TextRange) {
        let current_pc = self.get_pc_label();
        if current_pc > pc {
            self.pc.push_front(PC {
                label: current_pc,
                range: self.get_pc_expr_range(),
            });
        } else {
            self.pc.push_front(PC { label: pc, range });
        }
    }

    /// Pop the current level of the information flow state
    pub(crate) fn pop_pc(&mut self) {
        self.pc.pop_front();
    }

    pub(crate) fn get_label(&self, binding_id: BindingId) -> Option<Label> {
        return self.variable_map.get(&binding_id).cloned();
    }

    pub(crate) fn get_parameter_label(
        &self,
        function_binding_id: BindingId,
        index: usize,
    ) -> Option<Label> {
        if let Some(labels) = self.function_parameter_map.get(&function_binding_id) {
            if let Some(label) = labels.get(&index.to_string()) {
                return Some(label.clone());
            }
        }
        None
    }

    pub(crate) fn add_parameter_name(&mut self, binding_id: BindingId, name: &str, index: usize) {
        // Update the function parameter map with the parameter name
        // Get the currently indexed label from
        let label = self.get_parameter_label(binding_id, index);

        if let Some(label) = label {
            self.function_parameter_map
                .entry(binding_id)
                .or_insert_with(FxHashMap::default)
                .insert(name.to_string(), label);
        }
    }

    fn get_previous_line(locator: &Locator, range: TextRange) -> Option<TextRange> {
        let current_line = locator.line_range(range.start());
        if (current_line.start().to_u32()) == 0 {
            return None;
        }

        let previous_line = locator.line_range(TextSize::from(current_line.start().to_u32() - 1));
        Some(previous_line)
    }

    pub(crate) fn add_variable_label_binding(
        &mut self,
        binding_id: BindingId,
        range: TextRange,
        locator: &Locator,
        comment_ranges: &CommentRanges,
        binding_label: Option<Label>,
    ) {
        // Check for label from shadowed bindings
        // TODO: Implement inheritance from shadowed bindings
        // TODO: Declassification (invalid declassification check?)

        // Use the label if it is provided
        if let Some(label) = binding_label {
            self.variable_map.insert(binding_id, label);
            return;
        }

        // Read from comment
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
            return Some((label, *comment_range));
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
                return Some((label, *comment_range));
            }
        }
    }
    None
}

#[cfg(test)]
mod information_flow_state_tests {
    use ruff_python_ast::{identifier, Stmt, StmtAssign, StmtFunctionDef};
    use ruff_python_parser::{parse_program, tokenize, Mode};
    use ruff_python_semantic::ScopeId;

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
        let kind = BindingKind::Assignment;

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
                            &kind,
                            "",
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

        let kind = BindingKind::Assignment;
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
                                &kind,
                                "",
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
        let kind = BindingKind::Assignment;
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
                            &kind,
                            "",
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

    #[test]
    fn test_information_flow_state_function() {
        let source: &str = r#"
# iflabel fn ({secret}, {public}) {secret}
def help(a,b):
  # Checking internal run of the function using arg labels
  some_outer_secret_value = a # OK
  some_outer_public_value = a # FAIL a is secret
  return public # OK
  return secret # OK but not if return was "public"

secret = 1 # iflabel {secret}
public = 2 # iflabel {secret}

# Checking args
help(secret, public) # OK
help(public, secret) # Fail b has a public label

# Checking return
secret = help(secret, public) # OK
public = help(secret, public) # Fail public cannot be assigned a secret return value from help
"#;

        let tokens = tokenize(source, Mode::Module);
        let locator = Locator::new(source);
        let indexer = Indexer::from_tokens(&tokens, &locator);
        let comment_ranges = indexer.comment_ranges();
        let result = parse_program(source);
        let kind = BindingKind::FunctionDefinition(ScopeId::from(0u32));
        let mut state = InformationFlowState::new(&indexer, &locator);

        let mut id: BindingId = BindingId::from(0u32);

        match result {
            Ok(module) => {
                let stmts = module.body;
                for stmt in stmts {
                    match stmt {
                        Stmt::FunctionDef(StmtFunctionDef { range, name, .. }) => {
                            let binding_id: BindingId = id;
                            id = (id.as_u32() + 1).into();
                            state.add_variable_label_binding(
                                binding_id,
                                range,
                                &locator,
                                comment_ranges,
                                &kind,
                                &name.id,
                            );
                            state.add_variable_label_binding(
                                binding_id + 1,
                                range,
                                &locator,
                                comment_ranges,
                                &BindingKind::Argument,
                                "a",
                            );
                        }
                        _ => {}
                    }
                }
            }
            Err(_) => panic!("Failed to parse module"),
        }
        assert!(false);
    }
}
