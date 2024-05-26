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

use crate::checkers::{
    ast::Checker,
    information_flow::{
        helper::{get_label_for_expression, get_variable_label_by_name},
        label::Label,
    },
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
}

impl Violation for IFImplicitFunctionReturn {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Illegal implicit information flow. Defined return label: {} is less than returned variable {} with label: {}", self.defined_return_label.to_string(), self.return_expr, self.return_label.to_string())
    }
}

/// IF202
pub(crate) fn implicit_function_return(checker: &mut Checker, stmt: &Stmt) {
    if let Stmt::Return(StmtReturn { value, range }) = stmt {
        if let Some(value) = value {
            let return_label = get_label_for_expression(checker, value);

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

                if !(return_label <= defined_return_label) {
                    checker.diagnostics.push(Diagnostic::new(
                        IFImplicitFunctionReturn {
                            defined_return_label: defined_return_label.unwrap_or_default(),
                            return_label: return_label.unwrap_or_default(),
                            return_expr: checker.locator().slice(value.range()).to_string(),
                        },
                        range.clone(),
                    ));
                }
            }
        }
    }
}
