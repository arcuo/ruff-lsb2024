use ruff_python_ast::StmtAssign;
use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_text_size::Ranged;

use crate::checkers::ast::Checker;

/// ## What it does
/// Assert assignment to labels
///
/// ## Why is this bad?
/// ...
///
/// ## Example
/// ```python
/// ...
/// ```
///
/// Use instead:
/// ```python
/// ...
/// ```
#[violation]
pub struct UnauthorisedVariableAssign;
// {
//     var1: String,
//     label1: String,
//     var2: String,
//     label2: String
// }

// TODO: Add authorisation and variable information

impl Violation for UnauthorisedVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("Unauthorised assignment of variable")
    }
}

// TODO
/// IF001
pub(crate) fn unauthorised_variable_assign(checker: &mut Checker, assign: &StmtAssign) {
    if checker.semantic().in_async_context() {
        // if checker
        //     .semantic()
        if true {
            checker
                .diagnostics
                .push(Diagnostic::new(UnauthorisedVariableAssign, assign.range()));
        }
    }
}
