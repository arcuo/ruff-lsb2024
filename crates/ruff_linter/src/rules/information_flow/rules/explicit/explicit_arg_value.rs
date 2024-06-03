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
/// Check confidentiality of information flow in values set to function arguments for explicit flows.
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
pub struct IFExplicitArgument {
    expr_string: String,
    arg_label: Label,
    argname: String,
    defined_arg_label: Label,
    property: SecurityProperty,
}

impl Violation for IFExplicitArgument {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Invalid {} explicit argument flow: {}",
            self.property.to_string(),
            self.property.get_description(
                &self.argname,
                self.defined_arg_label.to_string(),
                &self.expr_string,
                self.arg_label.to_string(),
            )
        )
    }
}

/// IF202
pub(crate) fn check_explicit_arg_value(
    checker: &mut Checker,
    function_binding_id: BindingId,
    arg: &Expr,
    arg_index: usize,
    security_property: &SecurityProperty,
) {
    let (argname, defined_arg_label) = checker
        .information_flow()
        .get_parameter_label_by_index(function_binding_id, arg_index);

    let arg_label = get_label_for_expression(checker.semantic(), checker.information_flow(), arg)
        .unwrap_or(checker.information_flow().default_label());

    let defined_arg_label = defined_arg_label.unwrap_or(checker.information_flow().default_label());

    if arg_label == defined_arg_label {
        return;
    }

    let property = if arg_label < defined_arg_label {
        // arg_label is less trusted than defined_arg_label (integrity violation)
        SecurityProperty::Integrity
    } else if arg_label > defined_arg_label {
        // arg is more trusted than defined_arg_label (confidentiality violation)
        SecurityProperty::Confidentiality
    } else {
        // arg_label is in another branch than the defined_arg_label
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

    let expr_string = checker.locator().slice(arg.range()).to_string();

    let diagnostic = Diagnostic::new(
        IFExplicitArgument {
            expr_string,
            arg_label,
            argname: argname.to_string(),
            defined_arg_label,
            property: shown_property,
        },
        arg.range(),
    );
    checker.diagnostics.push(diagnostic);
}

pub(crate) fn check_explicit_keyword_value(
    checker: &mut Checker,
    function_binding_id: BindingId,
    kw: &Keyword,
    security_property: &SecurityProperty,
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

    let arg_label =
        get_label_for_expression(checker.semantic(), checker.information_flow(), &kw.value)
            .unwrap_or(checker.information_flow().default_label());

    if arg_label == defined_arg_label {
        return;
    }

    let property = if arg_label < defined_arg_label {
        // arg_label is less trusted than defined_arg_label (integrity violation)
        SecurityProperty::Integrity
    } else if arg_label > defined_arg_label {
        // pc is more trusted than defined_arg_label (confidentiality violation)
        SecurityProperty::Confidentiality
    } else {
        // arg_label is in another branch than the defined_arg_label
        SecurityProperty::Both
    };

    if security_property.skip_diagnostic(&property) {
        return;
    }

    let expr_string = checker.locator().slice(kw.value.range()).to_string();

    let diagnostic = Diagnostic::new(
        IFExplicitArgument {
            expr_string,
            argname: arg.as_str().to_string(),
            arg_label,
            defined_arg_label,
            property,
        },
        kw.value.range(),
    );
    checker.diagnostics.push(diagnostic);
}
