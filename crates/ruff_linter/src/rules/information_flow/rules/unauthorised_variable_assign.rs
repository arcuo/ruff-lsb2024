use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{Expr, Stmt};
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;

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
pub(crate) fn unauthorised_variable_assign(checker: &mut Checker, assign: &Stmt) {
    if is_unauthorised_assign_statement(checker, assign) {
        // Add diagnostics
        checker
            .diagnostics
            .push(Diagnostic::new(UnauthorisedVariableAssign, assign.range()));
    }
}

fn is_unauthorised_assign_statement(checker: &mut Checker, stmt: &Stmt) -> bool {
    // TODO: cleanup here
    // Get variable and value names
    if let Some((Some(variable_name), value_name)) = match stmt {
        Stmt::Assign(assign) => {
            if let variable_name = match assign.targets.first() {
                // TODO: multiple assignment, for now only check first target
                Some(Expr::Name(expr_name)) => Some(expr_name),
                _ => None,
            } {
                if let value_name = match assign.value.as_ref() {
                    Expr::Name(expr_name) => Some(expr_name), // Check for values in Tuples, Lists, Classes, etc.
                    _ => None,
                } {
                    Some((variable_name, value_name))
                } else {
                    Some((variable_name, None))
                }
            } else {
                None
            }
        }
        _ => None,
    } {
        // Get labels
        let variable_label = get_variable_label(checker, variable_name);
        let value_label = if value_name.is_some() {
            get_variable_label(checker, value_name.unwrap())
        } else {
            None
        };

        // No label for the value or it is public, then it is not unauthorised
        if value_label.is_none() || value_label.unwrap().is_public() {
            return false;
        }

        // Check information flow lattice

    }

    return false;
}
