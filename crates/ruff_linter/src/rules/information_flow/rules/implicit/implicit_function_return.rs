use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprName, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp, Stmt,
    StmtFunctionDef, StmtReturn,
};
use ruff_python_semantic::{Scope, ScopeKind};
use ruff_source_file::OneIndexed;
use ruff_text_size::TextRange;

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
/// Check confidentiality of information flow in variables returned by functions for implicit flows.
///
/// ## Why is this bad?
/// You do not want a function to return a variable with a higher label than the defined return label. Otherwise, it could lead to information leakage.
/// ...
///
/// ## Example
///
/// ```python
//  # iflabel fn (a: {alice}, b: {bob}, public: {}) {alice} <=== return label
/// def help(a,b, public):
///   alice_return = 1 # iflabel {alice}
///   public_return = 3 # iflabel {}
///   bob_return = 2 # iflabel {bob}
///   alice_bob_return = 4 # iflabel {alice, bob}
///
///   return alice_return # Succeed
///   return public_return # Succeed
///   return bob_return # Fail
///   return alice_bob_return # Fail
///```
#[violation]
pub struct IFImplicitFunctionReturn {
    defined_return_label: Label,
    return_expr: String,
    return_label: Label,
    property: SecurityProperty,
}

impl Violation for IFImplicitFunctionReturn {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Invalid {} implicit argument flow: {}",
            self.property.to_string(),
            self.property.get_description_return(
                &self.return_expr,
                self.return_label.to_string(),
                self.defined_return_label.to_string()
            )
        )
    }
}

/// IF202
pub(crate) fn implicit_function_return(
    checker: &mut Checker,
    stmt: &Stmt,
    security_property: &SecurityProperty,
) {
    if let Stmt::Return(StmtReturn { value, range }) = stmt {
        if let Some(value) = value {
            let return_label =
                get_label_for_expression(checker.semantic(), checker.information_flow(), value);

            // Get function label
            let ScopeKind::Function(StmtFunctionDef { name: fn_name, .. }) =
                checker.semantic().current_scope().kind
            else {
                return;
            };

            let Some(parent_scope) = checker
                .semantic()
                .first_non_type_parent_scope(checker.semantic().current_scope())
            else {
                unreachable!("Expected parent scope to be a function scope")
            };

            if let Some(fn_bid) = parent_scope.get(fn_name) {
                let defined_return_label = checker.information_flow().get_label(fn_bid);

                if return_label == defined_return_label {
                    return;
                }

                let property = if return_label < defined_return_label {
                    // return_label is less trusted than defined_return_label (integrity violation)
                    SecurityProperty::Integrity
                } else if return_label > defined_return_label {
                    // return_label is more trusted than defined_return_label (confidentiality violation)
                    SecurityProperty::Confidentiality
                } else {
                    // return_label is in another branch than the defined_return_label
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

                checker.diagnostics.push(Diagnostic::new(
                    IFImplicitFunctionReturn {
                        defined_return_label: defined_return_label
                            .unwrap_or(checker.information_flow().default_label()),
                        return_label: return_label
                            .unwrap_or(checker.information_flow().default_label()),
                        return_expr: checker.locator().slice(value.range()).to_string(),
                        property: shown_property,
                    },
                    range.clone(),
                ));
            }
        }
    }
}
