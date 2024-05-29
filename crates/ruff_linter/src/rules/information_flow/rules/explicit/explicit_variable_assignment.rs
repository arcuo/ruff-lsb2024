use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCall, ExprCompare, ExprDict, ExprIf,
    ExprList, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};

use crate::{
    checkers::{
        ast::Checker,
        information_flow::{
            helper::{get_label_for_expression, get_variable_label_by_name},
            label::Label,
        },
    },
    rules::information_flow::settings::SecurityProperty,
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
    value: String,
    value_label: Label,
    property: SecurityProperty,
}

impl Violation for IFExplicitVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Invalid {} explicit flow: {}",
            self.property.to_string(),
            self.property.get_description(
                &self.target,
                self.target_label.to_string(),
                &self.value,
                self.value_label.to_string()
            )
        )
    }
}

// TODO
/// IF101
pub(crate) fn check_if_assign_targets_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    value: &Expr,
    security_property: &SecurityProperty,
) {
    for target in targets {
        illegal_assign_target_statement(checker, target, value, security_property);
    }
}

/// IF101
/// T_ASSIGN_EXPLICIT: label(value) <= label(target) (not checking implicit flow)
pub(crate) fn illegal_assign_target_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
    security_property: &SecurityProperty,
) {
    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                illegal_assign_target_statement(checker, element, value, security_property);
            }
        }
        Expr::Name(target_name) => {
            let target_label = get_variable_label_by_name(
                checker.semantic(),
                checker.information_flow(),
                target_name,
            );

            if let Some(value_label) =
                get_label_for_expression(checker.semantic(), checker.information_flow(), value)
            {
                if value_label.is_public() {
                    return;
                }

                // Value is not public, check if label(target) >= label(value)
                if let Some(target_label) = target_label {
                    if value_label == target_label {
                        return;
                    }

                    let property = if value_label < target_label {
                        // Value is less trusted than target (integrity violation)
                        SecurityProperty::Integrity
                    } else if value_label > target_label {
                        // Value is more confidential than target
                        SecurityProperty::Confidentiality
                    } else if value_label != target_label {
                        // The labels are on different branches
                        SecurityProperty::Both
                    } else {
                        unreachable!()
                    };

                    if security_property.skip_diagnostic(&property) {
                        return;
                    }

                    let shown_property = if security_property.is_both() {
                        property
                    } else {
                        security_property.clone()
                    };

                    checker.diagnostics.push(Diagnostic::new(
                        IFExplicitVariableAssign {
                            target: target_name.id.clone(),
                            target_label,
                            value: checker.locator().slice(value.range()).to_string(),
                            value_label,
                            property: shown_property,
                        },
                        target.range(),
                    ));
                }
            }
        }
        _ => {}
    }
}
