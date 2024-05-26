use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprName, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp, Keyword,
    Parameter, Stmt, StmtFunctionDef, StmtReturn,
};
use ruff_python_semantic::{BindingId, Scope, ScopeKind};
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
/// Check confidentiality of information flow in values set to function arguments for implicit flows.
///
/// ## Why is this bad?
/// You do not want a function to receive a variable with a higher label than the defined argument label. Otherwise, it could lead to information leakage.
/// ...
///
/// ## Example
///
/// ```python
//  # iflabel fn (a: {alice}, p: {})
/// def help(a, p):
///   public_log(p) # public argument is fed to a public function
///
/// secret = 1 # iflabel {alice}
///
/// help(secret, 2) #  Succeed
/// help(2, secret) # Fail as secret is passed to public argument
///
///```
#[violation]
pub struct IFImplicitArgument {
    argname: String,
    arg_label: Label,
    defined_arg_label: Label,
}

impl Violation for IFImplicitArgument {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!("")
    }
}

/// IF202
pub(crate) fn check_implicit_arg_value(
    checker: &mut Checker,
    function_binding_id: BindingId,
    arg: &Expr,
    arg_index: usize,
) {
    let (argname, defined_arg_label) = checker
        .information_flow()
        .get_parameter_label_by_index(function_binding_id, arg_index)
        .unwrap_or_default();

    let arg_label = get_label_for_expression(checker, arg).unwrap_or_default();

    if defined_arg_label < arg_label {
        let diagnostic = Diagnostic::new(
            IFImplicitArgument {
                argname: argname.to_string(),
                arg_label,
                defined_arg_label,
            },
            arg.range(),
        );
        checker.diagnostics.push(diagnostic);
    }
}

pub(crate) fn check_implicit_keyword_value(
    checker: &mut Checker,
    function_binding_id: BindingId,
    kw: &Keyword,
) {
    let Some(arg) = &kw.arg else {
        return;
    };

    let Some(defined_arg_label) = checker
        .information_flow()
        .get_parameter_label_by_name(function_binding_id, arg.as_str())
    else {
        return;
    };

    let arg_label = get_label_for_expression(checker, &kw.value).unwrap_or_default();

    if defined_arg_label < arg_label {
        let diagnostic = Diagnostic::new(
            IFImplicitArgument {
                argname: arg.as_str().to_string(),
                arg_label,
                defined_arg_label,
            },
            kw.value.range(),
        );
        checker.diagnostics.push(diagnostic);
    }
}
