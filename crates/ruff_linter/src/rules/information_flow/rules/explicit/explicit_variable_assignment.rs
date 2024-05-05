use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::Expr;

use crate::checkers::{ast::Checker, information_flow::label::Label};

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
        format!(
            "Inconfidential assignment to more restrictive variable. Expression `{}` with label `{}` is being assigned to `{}` with label `{}`",
            self.var, self.var_label.to_string(), self.expr, self.expr_label.to_string()
        )
    }
}

// TODO
/// IF101
pub(crate) fn inconfidential_assign_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    value: &Expr,
) {
    if let Some(result) = is_inconfidential_assign_statement(checker, targets, value) {
        // Add diagnostics
        checker
            .diagnostics
            .push(Diagnostic::new(result, value.range()));
    }
}

/// Check if a variable assignment has correct information flow, in terms of confidentiality.
/// I.e. the variable label is more restrictive than the value label or the same.
fn is_inconfidential_assign_statement(
    checker: &mut Checker,
    targets: &[Expr],
    value: &Expr,
) -> Option<IFInconfidentialVariableAssign> {
    // Get variable and value names
    let Some(Expr::Name(variable_name)) = targets.first() else {
        return None;
    };

    let Expr::Name(value_name) = value else {
        return None;
    };

    // Get labels
    let variable_label = get_variable_label(checker, variable_name);
    let value_label = get_variable_label(checker, value_name);

    // No label for the variable or value, then it is not unauthorised
    if variable_label.is_none() || value_label.is_none() {
        return None;
    }

    if !variable_label
        .as_ref()
        .unwrap()
        .is_higher_in_lattice_path(value_label.as_ref().unwrap())
    {
        return Some(IFInconfidentialVariableAssign {
            var: variable_name.id.clone(),
            var_label: variable_label.unwrap(),
            expr: value_name.id.clone(),
            expr_label: value_label.unwrap(),
        });
    } else {
        return None;
    }
}
