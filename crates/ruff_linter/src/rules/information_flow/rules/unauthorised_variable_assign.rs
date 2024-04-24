use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::StmtAssign;
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
    // Use checker.indexer.comment_ranges ??
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

/// A label can indicate that the variable is either public or owned
/// 
/// Below are public
/// ```python
/// # iflabel public
/// a = 1
/// a = 2 # iflabel public
/// 
/// a = 3
/// ```
/// 
/// Below are owned
/// ```python
/// # iflabel {alice, []}
/// a = 1
/// a = 2 # iflabel {bob, []}
/// ```
enum LabelOwnership {
    Public,
    Owned,
}

struct VariableLabel {
    owner: Some(String),
    ownership: LabelOwnership,
    
}

/// Check comments above or inline for label, otherwise return public
fn check_for_label(checker: &mut Checker, assign: &StmtAssign) {}
