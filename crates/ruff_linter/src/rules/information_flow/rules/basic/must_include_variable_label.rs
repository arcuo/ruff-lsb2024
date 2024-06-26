use ruff_diagnostics::{Diagnostic, Edit, Fix, FixAvailability, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{Expr, ExprName, ExprTuple};
use ruff_source_file::Locator;
use ruff_text_size::TextRange;

use crate::checkers::{
        ast::Checker,
        information_flow::{information_flow_state::get_comment_label, label::Label},
    };

/// ## What it does
/// Check if variables have explicit labels.
///
/// ## Why is this bad?
/// Variables are by default public. This can introduce unintended information leakage.
/// The developer should explicitly define the label for the variable.
/// ...
///
/// ## Example
/// ```python
/// # BAD
/// public_var = ...
///
/// # GOOD
/// public_var = ...  # iflabel {}
/// ```
#[violation]
pub struct IFMustIncludeVariableLabel {
    var: String,
}

impl Violation for IFMustIncludeVariableLabel {
    const FIX_AVAILABILITY: FixAvailability = FixAvailability::Always;

    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
      "Missing variable label for `{}`. This can introduce unintended information leakage. Please add an explicit label to the variable `iflabel {{ ... }}` or `iflabel {{}}` for public.",
      self.var
    )
    }

    fn fix_title(&self) -> Option<String> {
        Some(format!(
            "Add explicit public label to the variable `{}`",
            self.var
        ))
    }
}

/// IF001
pub(crate) fn must_include_target_variable_label(
    checker: &mut Checker,
    target: &Expr,
    assign_range: &TextRange,
) {
    match target {
        Expr::Tuple(ExprTuple { elts, .. }) => {
            for element in elts {
                must_include_target_variable_label(checker, element, assign_range);
            }
        }
        Expr::Name(ExprName { id, .. }) => {
            let Some(binding_id) = checker.semantic().current_scope().get(id) else {
                return;
            };

            // If the target is a variable, check if it is a rebinding
            if checker
                .semantic()
                .current_scope()
                .shadowed_binding(binding_id)
                .is_none()
            {
                // Check if the variable has a label
                if get_comment_label(
                    target.range(),
                    checker.locator(),
                    checker.indexer().comment_ranges(),
                )
                .is_none()
                {
                    // Add diagnostics
                    let diagnostic = Diagnostic::new(
                        IFMustIncludeVariableLabel {
                            var: id.to_string(),
                        },
                        target.range(),
                    )
                    .with_fix(add_public_label_inline(*assign_range));

                    checker.diagnostics.push(diagnostic);
                }
            }
        }
        _ => {}
    }
}

/// IF001
pub(crate) fn must_include_targets_variable_label(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    assign_range: &TextRange,
) {
    for target in targets {
        must_include_target_variable_label(checker, target, assign_range);
    }
}

// IF001 fix
fn add_public_label_inline(assign_range: TextRange) -> Fix {
    Fix::safe_edit(Edit::insertion(
        " # iflabel {}".to_string(),
        assign_range.end(),
    ))
}
