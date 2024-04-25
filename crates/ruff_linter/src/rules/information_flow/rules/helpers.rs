use ruff_python_ast::ExprName;

use crate::checkers::{ast::Checker, information_flow::Label};

/// Fetch the label of a variable in the given scope
pub(super) fn get_variable_label(checker: &mut Checker, name: &ExprName) -> Option<Label> {
    // Get [BindingId]
    if let Some(binding_id) = checker.semantic().current_scope().get(name.id.as_str()) {
        // Get [Label] from information_flow
        if let Some(label) = checker.information_flow().get_label(binding_id) {
            return Some(label);
        }
    }

    None
}
