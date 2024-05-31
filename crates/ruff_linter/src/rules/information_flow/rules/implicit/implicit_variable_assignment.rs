use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprName, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp, Stmt,
};
use ruff_source_file::OneIndexed;
use ruff_text_size::{Ranged, TextRange};

use crate::{
    checkers::{
        ast::Checker,
        information_flow::{
            helper::{get_label_for_expression, get_variable_label_by_name},
            label::Label,
        },
    },
    rules::information_flow::SecurityProperty,
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
    pc_expr_line_number: OneIndexed,
    pc: Label,
    property: SecurityProperty,
}

impl Violation for IFImplicitVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Invalid {} implicit flow: {}",
            self.property.to_string(),
            self.property.get_description_pc(
                &self.target,
                self.target_label.to_string(),
                self.pc.to_string()
            )
        )
    }
}

// TODO
/// IF201
pub(crate) fn implicit_inconfidential_assign_targets_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    security_property: &SecurityProperty,
) {
    for target in targets {
        implicit_inconfidential_assign_target_statement(checker, target, security_property);
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
    security_property: &SecurityProperty,
) {
    // if pc is public, no need to check.
    if checker.information_flow().get_pc_label().is_public() {
        return;
    }

    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                implicit_inconfidential_assign_target_statement(
                    checker,
                    element,
                    security_property,
                );
            }
        }
        Expr::Name(name_target) => {
            let target_label = get_variable_label_by_name(
                checker.semantic(),
                checker.information_flow(),
                name_target,
            );
            let pc = checker.information_flow().get_pc_label();

            if let Some(target_label) = target_label {
                if pc == target_label {
                    return;
                }

                let property = if pc < target_label {
                    // pc is less trusted than target (integrity violation)
                    SecurityProperty::Integrity
                } else if pc > target_label {
                    // pc is more trusted than target (confidentiality violation)
                    SecurityProperty::Confidentiality
                } else {
                    // pc is in another branch than the target
                    SecurityProperty::Both
                };

                if security_property.skip_diagnostic(&property) {
                    return;
                }

                let shown_property = if security_property.is_both() {
                    property
                } else {
                    security_property.clone()
                };

                let stmt_range =
                    if let Stmt::Assign(assign) = checker.semantic().current_statement() {
                        assign.range()
                    } else {
                        target.range()
                    };

                let pc_expr_range = checker.information_flow().get_pc_expr_range();
                #[allow(deprecated)]
                checker.diagnostics.push(Diagnostic::new(
                    IFImplicitVariableAssign {
                        target: name_target.id.clone(),
                        target_label,
                        pc_expr_line_number: checker
                            .locator()
                            .compute_line_index(pc_expr_range.start()),
                        pc,
                        property: shown_property,
                    },
                    stmt_range,
                ));
            }
        }
        _ => {}
    }
}
