use ruff_python_ast::ExprName;

use crate::checkers::{ast::Checker, information_flow::label::Label};

/// Fetch the label of a variable in the given scope
pub(super) fn get_variable_label(checker: &mut Checker, name: &ExprName) -> Option<Label> {
    // Get shadowed [BindingId] from [Scope] if it exists. We only have to check shadowed bindings,
    // because otherwise the variable is new and does not have a label
    if let Some(binding_id) = checker.semantic().current_scope().get(name.id.as_str()) {
        if let Some(actual_binding_id) = checker
            .semantic()
            .current_scope()
            .shadowed_binding(binding_id)
        {
            // Get [Label] from information_flow
            if let Some(label) = checker.information_flow().get_label(actual_binding_id) {
                return Some(label)
            }
        } else {
            // Get [Label] from information_flow
            if let Some(label) = checker.information_flow().get_label(binding_id) {
                return Some(label);
            }
        }
    }

    None
}
