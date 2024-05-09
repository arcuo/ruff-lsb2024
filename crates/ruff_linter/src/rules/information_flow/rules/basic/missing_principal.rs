use anyhow::Result;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{Expr, ExprName, ExprTuple};
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};

use crate::{
    checkers::{
        ast::Checker,
        information_flow::{
            information_flow_state::read_variable_label_from_source, label::Label,
            principals::Principals,
        },
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
///
/// ## Fix
/// Add the missing principal to the principals list.
/// ```python
/// # ifprincipals {A, C}
/// ...
/// ```
#[violation]
pub struct IFMissingPrincipal {
    label_stmt: String,
    missing_principal: String,
    global_principals: Principals,
}

impl Violation for IFMissingPrincipal {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Missing principal `{}` in the principals list for `{}`",
            self.missing_principal, self.label_stmt
        )
    }

    fn fix_title(&self) -> Option<String> {
        Some(format!(
            "Add principal `{}` to the top of the file `# ifprincipals {{{}}}`", // TODO: Add principals via config?
            self.missing_principal,
            self.global_principals.principals.join(", ") + ", " + &self.missing_principal
        ))
    }
}

/// IF002
pub(crate) fn missing_principal_from_label(checker: &mut Checker, assign_range: TextRange) {
    let Some((label, comment_range)) = read_variable_label_from_source(
        assign_range,
        checker.locator(),
        checker.indexer().comment_ranges(),
    ) else {
        return; // No label found, so no principal to check
    };

    let global_principals = checker.information_flow().principals().clone();
    let label_principals = &label.principals;

    for principal in label_principals {
        if !global_principals.principals.contains(&principal) {
            {
                let global_principals: &Principals = &global_principals;
                // Find the range of the label
                let comment_text = &checker.locator().slice(comment_range);
                let principal_range =
                    match TryInto::<TextSize>::try_into(comment_text.find(principal).unwrap()) {
                        Ok(label_start_index) => {
                            let principal_range = TextRange::new(
                                comment_range.start() + label_start_index,
                                comment_range.start() + label_start_index + TextSize::of(principal),
                            );
                            principal_range
                        }
                        Err(_) => {
                            comment_range // If the principal is not found, then the range is the whole comment
                        }
                    };

                let diagnostic = Diagnostic::new(
                    IFMissingPrincipal {
                        label_stmt: label.to_string(),
                        missing_principal: principal.clone(),
                        global_principals: global_principals.clone(),
                    },
                    principal_range,
                );

                checker.diagnostics.push(diagnostic);
            };
        }
    }
}
