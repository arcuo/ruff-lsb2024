use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::Expr;
use ruff_text_size::TextRange;

use crate::checkers::{ast::Checker, information_flow::label::can_convert_label};

use super::helpers::get_variable_label;

/// ## What it does
/// Assert assignment to labels
///
/// ## Why is this bad?
/// ...
///
/// ## Example
/// ```python
/// ...
/// ```
///
/// Use instead:
/// ```python
/// ...
/// ```
#[violation]
pub struct UnauthorisedVariableAssign;
// {
//     var1: String,
//     label1: String,
//     var2: String,
//     label2: String
// }

// TODO: Add authorisation and variable information

impl Violation for UnauthorisedVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Unauthorised assignment of variable")
    }
}

// TODO
/// IF001
pub(crate) fn unauthorised_variable_assign(
    checker: &mut Checker,
    range: TextRange,
    targets: &Vec<Expr>,
    value: &Box<Expr>,
) {
    if is_unauthorised_assign_statement(checker, targets, value) {
        // Add diagnostics
        checker
            .diagnostics
            .push(Diagnostic::new(UnauthorisedVariableAssign, range));
    }
}

fn is_unauthorised_assign_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    value: &Box<Expr>,
) -> bool {
    // Get variable and value names
    let variable_name = match targets.first() {
        // TODO: multiple assignment, for now only check first target
        Some(Expr::Name(expr_name)) => expr_name,
        _ => return false, // This should not happen in assignments, but check either way
    };

    let value_name = match value.as_ref() {
        Expr::Name(expr_name) => expr_name, // Check name expressions
        // TODO:Check for values in Tuples, Lists, Classes, etc.
        _ => return false,
    };

    // Get labels
    let variable_label = get_variable_label(checker, variable_name);
    let value_label = get_variable_label(checker, value_name);

    // No label for the variable or value, then it is not unauthorised
    if variable_label.is_none() || value_label.is_none() {
        return false;
    }

    // Check information flow lattice, i.e. that the variable label can be converted
    // to the value label i.e. the variable label is more restrictive than the value label
    let is_authorised = can_convert_label(
        &variable_label.as_ref().unwrap(),
        &value_label.as_ref().unwrap(),
    );

    return is_authorised;
}
