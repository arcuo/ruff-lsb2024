use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCall, ExprCompare, ExprDict, ExprIf,
    ExprList, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};

use crate::checkers::{
    ast::Checker,
    information_flow::{
        helper::{get_label_for_expression, get_variable_label_by_name},
        label::Label,
    },
};

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
pub struct IFExplicitVariableAssign {
    target: String,
    target_label: Label,
    expr: String,
    expr_label: Label,
}

impl Violation for IFExplicitVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Illegal explicit assignment to more restrictive variable. Target `{}` with label `{}` is being assigned to `{}` with label `{}`",
            self.target, self.target_label.to_string(), self.expr, self.expr_label.to_string()
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
        illegal_assign_target_statement(checker, target, value);
    }
}

/// IF101
/// T_ASSIGN_EXPLICIT: label(value) <= label(target) (not checking implicit flow)
pub(crate) fn illegal_assign_target_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
) {
    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                illegal_assign_target_statement(checker, element, value);
            }
        }
        Expr::Name(target_name) => {
            let target_label = get_variable_label_by_name(checker, target_name);

            if let Some(value_label) = get_label_for_expression(checker, value) {
                if value_label.is_public() {
                    return;
                }

                // Value is not public, check if label(target) >= label(value)
                if let Some(target_label) = target_label {
                    if !(value_label <= target_label) {
                        checker.diagnostics.push(Diagnostic::new(
                            IFExplicitVariableAssign {
                                target: target_name.id.clone(),
                                target_label,
                                expr: checker.locator().slice(value.range()).to_string(),
                                expr_label: value_label,
                            },
                            target.range(),
                        ));
                    }
                    
                }
            }
        }
        _ => {}
    }
}
