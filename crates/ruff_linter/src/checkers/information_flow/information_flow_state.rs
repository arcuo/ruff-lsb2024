use lazy_static::lazy_static;
use regex::Regex;
use ruff_python_ast::Stmt;
use rustc_hash::FxHashMap;
use std::collections::VecDeque;

use ruff_python_index::Indexer;
use ruff_python_semantic::{BindingId, BindingKind, SemanticModel};
use ruff_python_trivia::CommentRanges;
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};

use super::{
    helper::get_label_from_statement,
    label::{FunctionLabel, Label},
    principals::{initiate_principals, Principals},
};

lazy_static! {
    static ref EXTRACT_PREVIOUS_LINE: Regex = Regex::new(r"^\s*#\s*iflabel.*").unwrap();
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PC {
    /// The current PC label
    label: Label,
    /// The range of the statement where the PC is set
    range: TextRange,
}

impl Default for PC {
    fn default() -> Self {
        Self {
            label: Label::new_public(),
            range: TextRange::default(),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct FunctionDefinition {
    /// The return label of the function
    return_label: Option<Label>,
    /// The parameter labels of the function
    parameter_labels: FxHashMap<String, Label>,
    /// Function body for label inference
    body: Vec<Stmt>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VariableMap {
    variable_map: FxHashMap<BindingId, Label>,
    pub(crate) return_label: Option<Label>,
}

impl VariableMap {
    fn get(&self, binding_id: &BindingId) -> Option<&Label> {
        return self.variable_map.get(binding_id);
    }

    fn insert(&mut self, binding_id: BindingId, label: Label) {
        self.variable_map.insert(binding_id, label);
    }

    pub(crate) fn update_return_label(&mut self, label: Label) {
        if let Some(return_label) = self.return_label.clone() {
            self.return_label = Some(return_label + label);
        } else {
            self.return_label = Some(label);
        }
    }
}

/// State of the information flow
#[derive()]
pub(crate) struct InformationFlowState {
    /// The current principles of the program, e.g. ['alice', 'bob']
    #[allow(dead_code)]
    principals: Principals,
    /// The current scope level queue. The level is updated according to the scope by popping and
    pc: VecDeque<PC>,
    /// Map from variable name to
    variable_map_scopes: VecDeque<VariableMap>,
    /// Map from function name to parameter label
    function_map: FxHashMap<BindingId, FunctionDefinition>,
}

impl InformationFlowState {
    pub(crate) fn push_variable_scope(&mut self) {
        self.variable_map_scopes.push_front(VariableMap::default());
    }

    pub(crate) fn pop_variable_scope(&mut self) {
        self.variable_map_scopes.pop_front();
    }

    pub(crate) fn new(indexer: &Indexer, locator: &Locator) -> Self {
        let vm = VariableMap::default();
        Self {
            principals: initiate_principals(indexer, locator),
            variable_map_scopes: VecDeque::from([vm]),
            pc: VecDeque::default(),
            function_map: FxHashMap::default(),
        }
    }

    pub(crate) fn default_label(&self) -> Label {
        Label::new(self.principals.to_vec())
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
        for scope in self.variable_map_scopes.iter() {
            if let Some(label) = scope.get(&binding_id) {
                return Some(label.clone());
            }
        }
        None
    }

    pub(crate) fn get_return_label(&self, binding_id: BindingId) -> Option<Label> {
        if let Some(FunctionDefinition { return_label, .. }) = self.function_map.get(&binding_id) {
            return return_label.clone();
        }
        None
    }

    pub(crate) fn get_parameter_label_by_name(
        &self,
        function_binding_id: BindingId,
        parameter_name: &str,
    ) -> Option<Label> {
        if let Some(FunctionDefinition {
            parameter_labels, ..
        }) = self.function_map.get(&function_binding_id)
        {
            if let Some(label) = parameter_labels.get(parameter_name) {
                return Some(label.clone());
            }
        }
        None
    }

    pub(crate) fn get_fn_parameter_name(
        &self,
        fn_binding_id: BindingId,
        index: usize,
    ) -> Option<String> {
        if let Some(FunctionDefinition {
            parameter_labels, ..
        }) = self.function_map.get(&fn_binding_id)
        {
            if let Some(name) = parameter_labels.keys().nth(index) {
                return Some(name.clone());
            }
        }

        None
    }

    pub(crate) fn get_fn_body(&self, fn_binding_id: BindingId) -> Option<&Vec<Stmt>> {
        if let Some(FunctionDefinition { body, .. }) = self.function_map.get(&fn_binding_id) {
            return Some(body);
        }
        None
    }

    pub(crate) fn variable_map(&mut self) -> &mut VariableMap {
        self.variable_map_scopes.front_mut().unwrap()
    }

    /// Get the name and label of the parameter by index
    pub(crate) fn get_parameter_label_by_index(
        &self,
        function_binding_id: BindingId,
        index: usize,
    ) -> (String, Option<Label>) {
        if let Some(FunctionDefinition {
            parameter_labels, ..
        }) = self.function_map.get(&function_binding_id)
        {
            if let Some((name, label)) = parameter_labels.iter().nth(index) {
                return (name.clone(), Some(label.clone()));
            }
        }
        (String::new(), None)
    }

    /// Add a information flow label to the binding_id in the variable map
    pub(crate) fn add_binding_label(
        &mut self,
        kind: BindingKind,
        binding_id: BindingId,
        range: TextRange,
        locator: &Locator,
        comment_ranges: &CommentRanges,
        semantic: &SemanticModel,
    ) {
        match kind {
            // Ignored bindings
            BindingKind::Export(_)
            | BindingKind::Builtin
            | BindingKind::Argument // Arguments are handled separately in `visit_deferred_functions`.
            | BindingKind::TypeParam
            | BindingKind::ConditionalDeletion(_)
            | BindingKind::Deletion
            | BindingKind::BoundException
            | BindingKind::UnboundException(_)
            | BindingKind::ClassDefinition(_) => {},

            // Handled bindings
            BindingKind::Annotation
            | BindingKind::NamedExprAssignment
            | BindingKind::Assignment
            | BindingKind::LoopVar
            | BindingKind::WithItemVar
            | BindingKind::Global
            | BindingKind::Nonlocal(_) => {
                self.add_variable_label_binding(
                    binding_id,
                    range,
                    locator,
                    comment_ranges,
                    semantic,
                );
            }

            // Function bindings, do nothing, as this is handled manually in function definitions
            BindingKind::FunctionDefinition(_) => {},

            // todos
            BindingKind::FromImport(_)
            | BindingKind::Import(_)
            | BindingKind::FutureImport
            | BindingKind::SubmoduleImport(_) => {
                // TODO: Add information flow for imports.
            }
            BindingKind::ComprehensionVar => {
                // TODO for comprehension var
            },
        }
    }

    /// Add a variable label binding to the variable map
    /// Use [`InformationFlowState::add_binding_label`] instead
    pub(crate) fn add_variable_label_binding(
        &mut self,
        binding_id: BindingId,
        range: TextRange,
        locator: &Locator,
        comment_ranges: &CommentRanges,
        semantic: &SemanticModel,
    ) {
        // Check for label from shadowed bindings

        // Read from comment
        if let Some((label, _)) = get_comment_text(range, locator, comment_ranges) {
            if let Ok(label) = label.as_str().parse::<Label>() {
                self.variable_map().insert(binding_id, label);
                return;
            }
        }

        // If there exists no label for the variable, get label from value expression
        if semantic
            .shadowed_bindings(semantic.scope_id, binding_id)
            .any(|bid| self.get_label(bid.shadowed_id()).is_some())
        {
            return;
        }

        if let Some(label) = get_label_from_statement(semantic, self, semantic.current_statement())
        {
            self.variable_map().insert(binding_id, label);
            return;
        }
    }

    /// Add a function variable label binding to the variable map
    /// Use [`InformationFlowState::add_binding_label`] instead
    pub(crate) fn add_function_variable_label_binding(
        &mut self,
        function_binding_id: BindingId,
        body: Vec<Stmt>,
        range: TextRange,
        locator: &Locator,
        comment_ranges: &CommentRanges,
    ) {
        // Check for label from shadowed bindings
        // TODO: Implement inheritance from shadowed bindings

        // Instantiate function map
        let function_map = self
            .function_map
            .entry(function_binding_id)
            .or_insert_with(FunctionDefinition::default);

        function_map.body = body;

        // Read labels from comment
        if let Some((fn_label, _)) = get_comment_function_label(range, locator, comment_ranges) {
            // Insert return label into variable map based on binding_id
            self.variable_map_scopes
                .front_mut()
                .unwrap()
                .insert(function_binding_id, fn_label.return_label.clone());

            function_map.return_label = Some(fn_label.return_label);

            for (name, label) in fn_label.argument_labels.iter() {
                function_map
                    .parameter_labels
                    .insert(name.clone(), label.clone());
            }
            return;
        }
    }

    pub(crate) fn add_parameter_name_variable_label_binding(
        &mut self,
        function_binding_id: BindingId,
        parameter_binding_id: BindingId,
        parameter_name: &str,
    ) {
        // Check for label from shadowed bindings
        // TODO: Implement inheritance from shadowed bindings
        // TODO: Declassification (invalid declassification check?)

        // Insert arguments into variable map based on binding_id and parameter name
        if let Some(function_label) =
            self.get_parameter_label_by_name(function_binding_id, &parameter_name)
        {
            self.variable_map()
                .insert(parameter_binding_id, function_label);
        } else {
            // No label comment, add public label
            self.variable_map()
                .insert(parameter_binding_id, Label::new_public());
        }
    }

    pub(crate) fn principals(&self) -> &Principals {
        &self.principals
    }
}

pub(crate) fn get_comment_variable_label(
    range: TextRange,
    locator: &Locator,
    comment_ranges: &CommentRanges,
) -> Option<(Label, TextRange)> {
    if let Some((label, comment_range)) = get_comment_text(range, locator, comment_ranges) {
        if let Ok(label) = label.as_str().parse::<Label>() {
            return Some((label, comment_range));
        }
    }
    None
}

pub(crate) fn get_comment_function_label(
    range: TextRange,
    locator: &Locator,
    comment_ranges: &CommentRanges,
) -> Option<(FunctionLabel, TextRange)> {
    if let Some((label, comment_range)) = get_comment_text(range, locator, comment_ranges) {
        if let Ok(label) = label.as_str().parse::<FunctionLabel>() {
            return Some((label, comment_range));
        }
    }
    None
}

fn get_previous_line(locator: &Locator, range: TextRange) -> Option<TextRange> {
    let current_line = locator.line_range(range.start());
    if (current_line.start().to_u32()) == 0 {
        return None;
    }

    let previous_line = locator.line_range(TextSize::from(current_line.start().to_u32() - 1));
    if let Some(_) = EXTRACT_PREVIOUS_LINE.captures(locator.slice(previous_line)) {
        return Some(previous_line);
    }
    None
}

/// Get the comment text inline or line above
/// and the [`TextRange`] of the label
pub(crate) fn get_comment_text(
    range: TextRange,
    locator: &Locator,
    comment_ranges: &CommentRanges,
) -> Option<(String, TextRange)> {
    // Find comment on same line
    let line_range = locator.line_range(range.start());
    if let Some(inline_comment_range) = comment_ranges.comments_in_range(line_range).first() {
        let comment_text = locator.slice(inline_comment_range).replace('#', "");
        return Some((comment_text, inline_comment_range.clone()));
    }

    // Find comment on previous line if it exists
    if let Some(previous_line_range) = get_previous_line(locator, range) {
        let comment_text = locator.slice(previous_line_range).replace('#', "");
        return Some((comment_text, previous_line_range.clone()));
    };

    None
}
