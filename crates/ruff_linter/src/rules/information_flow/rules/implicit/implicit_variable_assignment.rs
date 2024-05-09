use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
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
pub struct IFImplicitInconfidentialVariableAssign {
    var: String,
    var_label: Label,
    expr: String,
    expr_label: Label,
    pc: Label,
}

impl Violation for IFImplicitInconfidentialVariableAssign {
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
    value: &Expr,
) {
    for target in targets {
        implicit_inconfidential_assign_target_statement(checker, target, value);
    }
}

/// IF201
pub(crate) fn implicit_inconfidential_assign_target_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
) {
    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                implicit_inconfidential_assign_target_statement(checker, element, value);
            }
        }
        Expr::Name(_) => {
           todo!()
        }
        _ => {}
    }
}
