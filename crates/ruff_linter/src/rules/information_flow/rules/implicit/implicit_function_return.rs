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
    return_label: Label,
}

impl Violation for IFImplicitFunctionReturn {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("") // TODO
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

            if let Some(fn_bid) = checker.semantic().current_scope().get(fn_name) {
                let defined_return_label = checker.information_flow().get_label(fn_bid);

                if return_label > defined_return_label {
                    checker.diagnostics.push(Diagnostic::new(
                        IFImplicitFunctionReturn {
                            defined_return_label,
                            return_label,
                        },
                        range.clone(),
                    ));
                }
            }
        }
    }
}
