use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};

use crate::checkers::{ast::Checker, information_flow::{helper::{get_label_for_expression, get_variable_label_by_name}, label::Label}};

/// ## What it does
/// Check confidentiality of information flow in variable assignments.
///
/// ## Why is this bad?
/// Public variables or variables with labels that are lower in the information flow lattice cannot flow up in the lattice
/// to the value being assigned to them. Due to the fact that they are not trusted to hold the sensitive information by their definition.
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
pub(crate) fn inconfidential_assign_targets_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    value: &Expr,
) {
    for target in targets {
        inconfidential_assign_target_statement(checker, target, value);
    }
}

/// IF101
pub(crate) fn inconfidential_assign_target_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
) {
    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                inconfidential_assign_target_statement(checker, element, value);
            }
        }
        Expr::Name(_) => {
            if let Some(result) = is_inconfidential_assign_statement(checker, target, value) {
                // Add diagnostics
                checker
                    .diagnostics
                    .push(Diagnostic::new(result, target.range()));
            }
        }
        _ => {}
    }
}

/// Check if a variable assignment has correct information flow, in terms of confidentiality.
/// I.e. the variable label is more restrictive than the value label or the same.
fn is_inconfidential_assign_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
) -> Option<IFInconfidentialVariableAssign> {
    // Get variable and value names
    let Expr::Name(variable_name) = target else {
        return None;
    };

    // TODO: Handle multiple targets

    // Get labels
    let variable_label = get_variable_label_by_name(checker, variable_name);
    let value_label = get_label_for_expression(checker, value);

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
            expr: checker.locator().full_lines(value.range()).to_string(),
            expr_label: value_label.unwrap(),
        });
    } else {
        return None;
    }
}
