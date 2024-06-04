use anyhow::Result;
use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{Expr, ExprName, ExprTuple};
use ruff_source_file::Locator;
use ruff_text_size::{TextRange, TextSize};

use crate::checkers::{
    ast::Checker,
    information_flow::{
        information_flow_state::{get_comment_function_label, get_comment_variable_label},
        label::{FunctionLabel, Label},
        principals::Principals,
    },
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

///IF002 Functions
pub(crate) fn missing_principal_from_function_label(
    checker: &mut Checker,
    function_range: TextRange,
) {
    let Some((fn_label, comment_range)) = get_comment_function_label(
        function_range,
        checker.locator(),
        checker.indexer().comment_ranges(),
    ) else {
        return; // No label found, so no principal to check
    };

    let global_principals = checker.information_flow().principals().clone();
    let return_label_principals = &fn_label.return_label.principals;

    for principal in return_label_principals {
        check_label_missing(
            checker,
            &global_principals,
            fn_label.to_string(),
            principal,
            comment_range,
        );
    }

    for (_, arg) in &fn_label.argument_labels {
        let arg_label_principals = &arg.principals;
        for principal in arg_label_principals {
            check_label_missing(
                checker,
                &global_principals,
                fn_label.to_string(),
                principal,
                comment_range,
            );
        }
    }
}

/// IF002 Assign
pub(crate) fn missing_principal_from_assign_label(checker: &mut Checker, assign_range: TextRange) {
    let Some((label, comment_range)) = get_comment_variable_label(
        assign_range,
        checker.locator(),
        checker.indexer().comment_ranges(),
    ) else {
        return; // No label found, so no principal to check
    };

    let global_principals = checker.information_flow().principals().clone();
    let label_principals = &label.principals;

    for principal in label_principals {
        check_label_missing(
            checker,
            &global_principals,
            label.to_string(),
            principal,
            comment_range,
        );
    }
}

fn check_label_missing(
    checker: &mut Checker,
    global_principals: &Principals,
    label_string: String,
    principal: &String,
    comment_range: TextRange,
) {
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
                    label_stmt: label_string,
                    missing_principal: principal.clone(),
                    global_principals: global_principals.clone(),
                },
                principal_range,
            )
            .with_fix(add_principle(checker, principal));

            checker.diagnostics.push(diagnostic);
        };
    }
}

// IF002 fix add missing principal
fn add_principle(checker: &mut Checker, missing_principal: &String) -> Fix {
    let Some(global_principals_range) = checker.information_flow().principals().range else {
        // Add principals to the top of the file
        return Fix::unsafe_edit(Edit::insertion(
            format!("# ifprincipals {{{}}}\n", missing_principal),
            checker.locator().contents_start(),
        ));
    };

    // Add principal to the existing principals list
    let Ok(mut current_principals) = checker
        .locator()
        .slice(global_principals_range)
        .parse::<Principals>()
    else {
        return Fix::unsafe_edit(Edit::insertion(
            format!("# ifprincipals {{{}}}\n", missing_principal),
            checker.locator().contents_start(),
        ));
    };

    current_principals.add_principle(missing_principal);

    Fix::unsafe_edit(Edit::range_replacement(
        format!("# {}", current_principals.to_string()),
        global_principals_range,
    ))
}
