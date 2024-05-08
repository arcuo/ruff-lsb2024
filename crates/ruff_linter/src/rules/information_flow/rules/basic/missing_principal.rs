use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{Expr, ExprName, ExprTuple};
use ruff_source_file::Locator;
use ruff_text_size::TextRange;

use crate::{
    checkers::{
        ast::Checker,
        information_flow::{information_flow_state::read_variable_label_from_source, label::Label},
    },
    rules::information_flow::rules::helpers::get_variable_label_by_name,
};

/// ## What it does
/// Check if principal added in label is missing from the principals list.
///
/// ## Why is this bad?
/// Only use principals that are defined in the principals list.
/// ...
///
/// ## Example
/// ```python
/// # BAD
/// # ifprincipals {A}
/// public_var = ... # iflabel {C}
///
/// # GOOD
/// public_var = ...  # iflabel {A}
///
/// ```
#[violation]
pub struct IFMissingPrincipal {
    var: String,
}

impl Violation for IFMissingPrincipal {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        todo!();
        format!(
      "Missing variable label for `{}` Variables are by default public. This can introduce unintended information leakage. Please add an explicit label to the variable `iflabel {{ ... }}` or `iflabel {{}}` for public.",
      self.var
    )
    }

    fn fix_title(&self) -> Option<String> {
        todo!();
        Some(format!(
            "Add explicit public label to the variable `{}`",
            self.var
        ))
    }
}

/// IF002
pub(crate) fn missing_principal_from_label(
    checker: &mut Checker,
    target: &Expr,
    assign_range: &TextRange,
) {
}

// IF002 fix add missing principal
fn add_principle(assign_range: TextRange) -> Fix {
    todo!();
    // Fix::safe_edit(Edit::insertion(
    //     " # iflabel {}".to_string(),
    //     assign_range.end(),
    // ))
}
