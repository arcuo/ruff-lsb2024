use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::Expr;

use crate::checkers::{
    ast::Checker,
    information_flow::label::{can_convert_label, Label},
};

use super::helpers::get_variable_label;

/// ## What it does
/// Check confidentiality of information flow in variable assignments.
///
/// ## Why is this bad?
/// Public variables or variables with labels that are cannot flow in the information flow lattice
/// to the value being assigned to them, are not trusted to hold the sensitive information by their definition.
/// ...
///
/// ## Example
/// ```python
/// public_var = ...  # iflabel {}
/// secret_var = ...  # iflabel {secret}
///
/// public_var = secret_var  # Label violation as {secret} -> {} is not allowed
/// ```
#[violation]
pub struct IFInconfidentialVariableAssign {
    var: String,
    var_label: Label,
    expr: String,
    expr_label: Label,
}

impl Violation for IFInconfidentialVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Unauthorised assignment of variable")
    }
}

// TODO
/// IF001
pub(crate) fn inconfidential_assign_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    value: &Expr,
) {
    if is_inconfidential_assign_statement(checker, targets, value) {
        // Add diagnostics
        checker.diagnostics.push(Diagnostic::new(
            IFInconfidentialVariableAssign {
                var: "var".to_string(), // TODO: Get variable name
                var_label: Label::default(), // TODO: Get label
                expr: "expr".to_string(), // TODO: Get expression string
                expr_label: Label::default(), // TODO: Get label
            },
            value.range(),
        ));
    }
}

/// Check if a variable assignment has correct information flow, in terms of confidentiality.
/// I.e. the variable label is more restrictive than the value label or the same.
fn is_inconfidential_assign_statement(
    checker: &mut Checker,
    targets: &[Expr],
    value: &Expr,
) -> bool {
    // Get variable and value names
    let Some(Expr::Name(variable_name)) = targets.first() else {
        return false
    };

    let Expr::Name(value_name) = value else {
        return false
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
        value_label.as_ref().unwrap(),
        variable_label.as_ref().unwrap(),
    );

    is_authorised
}
