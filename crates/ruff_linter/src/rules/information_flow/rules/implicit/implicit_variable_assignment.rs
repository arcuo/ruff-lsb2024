use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprName, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};

use crate::checkers::{
    ast::Checker,
    information_flow::{
        helper::{get_label_for_expression, get_variable_label_by_name},
        label::Label,
    },
};

/// ## What it does
/// Check confidentiality of information flow in variable assignments for implicit flows.
///
/// ## Why is this bad?
/// Implicit flows from branching that stems from if-statements can lead to information leakage of sensitive data.
/// ...
///
/// ## Example
///
/// ```python
/// secret_var = ...  # iflabel {secret}
/// public_var = ...  # iflabel {}
///
/// if secret_var:
///    public_var = 1 # BAD
///
/// ```
#[violation]
pub struct IFImplicitVariableAssign {
    target: String,
    target_label: Label,
    pc_expr: String,
    pc: Label,
}

impl Violation for IFImplicitVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("")
    }
}

// TODO
/// IF201
pub(crate) fn implicit_inconfidential_assign_targets_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
) {
    // if pc is public, no need to check.
    if checker.information_flow().get_pc_label().is_public() {
        return;
    }

    for target in targets {
        implicit_inconfidential_assign_target_statement(checker, target);
    }
}

/// IF201
/// T_ASSIGN_IMPLICIT: max(pc | label(value)) <= label(target)
///
/// Explicit is handled by the explicit_variable_assignment.rs so we only check for implicit flows here i.e. pc <= label(target)
///
/// Check that the max between the value and the target is less than or equal to the target label.
/// E.g. if the target has public but either the value or the target has secret, then it is a violation.
pub(crate) fn implicit_inconfidential_assign_target_statement(
    checker: &mut Checker,
    target: &Expr,
) {
    // if pc is public, no need to check.
    if checker.information_flow().get_pc_label().is_public() {
        return;
    }

    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                implicit_inconfidential_assign_target_statement(checker, element);
            }
        }
        Expr::Name(name_target) => {
            let target_label = get_variable_label_by_name(checker, name_target);
            let pc = checker.information_flow().get_pc_label();

            if let Some(target_label) = target_label {
                if pc > target_label {
                    let pc_expr_range = checker.information_flow().get_pc_expr_range();
                    checker.diagnostics.push(Diagnostic::new(
                        IFImplicitVariableAssign {
                            target: name_target.id.clone(),
                            target_label,
                            pc_expr: checker.locator().slice(pc_expr_range).to_string(),
                            pc,
                        },
                        target.range(),
                    ));
                }
            }
        }
        _ => {}
    }
}
