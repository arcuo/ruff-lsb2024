use ruff_diagnostics::{Diagnostic, Violation};
use ruff_macros::{derive_message_formats, violation};
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCompare, ExprDict, ExprIf, ExprList,
    ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};

use crate::checkers::{ast::Checker, information_flow::label::Label};

use super::helpers::get_variable_label_by_name;

/// ## What it does
/// Check confidentiality of information flow in variable assignments.
///
/// ## Why is this bad?
/// Public variables or variables with labels that are lower in the information flow lattice cannot flow up in the lattice
/// to the value being assigned to them. Due to the fact that they are not trusted to hold the sensitive information by their definition.
/// ...
///
/// ## Example
/// ```python
/// public_var = ...  # iflabel {}
/// secret_var = ...  # iflabel {secret}
///
/// public_var = secret_var  # Label violation as {secret} -> {} is not allowed
/// ```
#[violation]
pub struct IFInconfidentialVariableAssign {
    var: String,
    var_label: Label,
    expr: String,
    expr_label: Label,
}

impl Violation for IFInconfidentialVariableAssign {
    #[derive_message_formats]
    fn message(&self) -> String {
        format!(
            "Inconfidential assignment to more restrictive variable. Expression `{}` with label `{}` is being assigned to `{}` with label `{}`",
            self.var, self.var_label.to_string(), self.expr, self.expr_label.to_string()
        )
    }
}

// TODO
/// IF101
pub(crate) fn inconfidential_assign_targets_statement(
    checker: &mut Checker,
    targets: &Vec<Expr>,
    value: &Expr,
) {
    if let Some(target) = targets.first() {
        // TODO Handle multiple targets
        if let Some(result) = is_inconfidential_assign_statement(checker, target, value) {
            // Add diagnostics
            checker
                .diagnostics
                .push(Diagnostic::new(result, value.range()));
        }
    }
}

/// IF101
pub(crate) fn inconfidential_assign_target_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
) {
    if let Some(result) = is_inconfidential_assign_statement(checker, target, value) {
        // Add diagnostics
        checker
            .diagnostics
            .push(Diagnostic::new(result, value.range()));
    }
}

fn get_most_restrictive_label_from_list_of_expressions(
    checker: &mut Checker,
    expressions: &Vec<Expr>,
) -> Option<Label> {
    let mut curr_label: Option<Label> = None;
    for expr in expressions {
        if let Some(expr_label) = get_label_for_expression(checker, &expr) {
            if let Some(label) = curr_label.clone() {
                if expr_label.is_higher_in_lattice_path(&label) {
                    curr_label = Some(expr_label);
                }
            } else {
                curr_label = Some(expr_label);
            }
        }
    }

    if let Some(curr_label) = curr_label.clone() {
        return Some(curr_label.clone());
    }

    None
}

fn get_higher_of_two_labels(label1: Option<Label>, label2: Option<Label>) -> Option<Label> {
    if label1.is_none() {
        return label2;
    } else if label2.is_none() {
        return label1;
    } else {
        return if label1
            .as_ref()
            .unwrap()
            .is_higher_in_lattice_path(label2.as_ref().unwrap())
        {
            label1
        } else {
            label2
        };
    }
}

/// Get the label of an expression.
/// Returns the most restrictive label found in the expression
/// TODO: Return the range as well and name for a more detailed error message
fn get_label_for_expression(checker: &mut Checker, expr: &Expr) -> Option<Label> {
    match expr {
        Expr::Name(name) => get_variable_label_by_name(checker, name), // Get label from variable
        Expr::Subscript(ExprSubscript { slice, .. }) => get_label_for_expression(checker, slice),
        Expr::UnaryOp(ExprUnaryOp { operand, .. }) => get_label_for_expression(checker, operand),
        Expr::Named(ExprNamed {
            target: left,
            value: right,
            ..
        })
        | Expr::BinOp(ExprBinOp { left, right, .. }) => {
            let left_label = get_label_for_expression(checker, left);
            let right_label = get_label_for_expression(checker, right);

            // Return the label that is more restrictive
            get_higher_of_two_labels(left_label, right_label)
        }

        Expr::Slice(ExprSlice {
            lower, upper, step, ..
        }) => {
            let lower_label = if lower.is_none() {
                None
            } else {
                get_label_for_expression(checker, lower.as_ref().unwrap())
            };
            let upper_label = if upper.is_none() {
                None
            } else {
                get_label_for_expression(checker, upper.as_ref().unwrap())
            };
            let step_label = if step.is_none() {
                None
            } else {
                get_label_for_expression(checker, step.as_ref().unwrap())
            };

            let return_label = get_higher_of_two_labels(lower_label, upper_label);
            get_higher_of_two_labels(return_label, step_label)
        }

        // Expressions with dynamic number of values (i.e. a vector of expressions)
        Expr::BoolOp(ExprBoolOp { values, .. })
        | Expr::Tuple(ExprTuple { elts: values, .. })
        | Expr::List(ExprList { elts: values, .. })
        | Expr::Set(ExprSet { elts: values, .. }) => {
            get_most_restrictive_label_from_list_of_expressions(checker, values)
        }

        Expr::Dict(ExprDict { items, .. }) => {
            let values: &Vec<Expr> = &items
                .iter()
                .map(|item| item.value.clone())
                .collect::<Vec<_>>();
            get_most_restrictive_label_from_list_of_expressions(checker, values)
        }

        Expr::Attribute(ExprAttribute { value, .. }) => {
            // Get label from object
            get_label_for_expression(checker, value)
        } // TODO: For now we only handle the object label. Handle attribute individual attributes expressions (i.e. attributes from classes)

        Expr::Compare(ExprCompare {
            left, comparators, ..
        }) => {
            let left_label = get_label_for_expression(checker, left);
            let right_label =
                get_most_restrictive_label_from_list_of_expressions(checker, &comparators.to_vec());

            get_higher_of_two_labels(left_label, right_label)
        }
        Expr::If(ExprIf {
            body, test, orelse, ..
        }) => {
            let test_label = get_label_for_expression(checker, test);
            let body_label = get_label_for_expression(checker, body);
            let orelse_label = get_label_for_expression(checker, orelse);

            let return_label = get_higher_of_two_labels(body_label, orelse_label);
            get_higher_of_two_labels(return_label, test_label)
        }

        // Comprehensions
        Expr::ListComp(_) => None, // TODO: Handle list comprehensions
        Expr::SetComp(_) => None,  // TODO: Handle set comprehensions
        Expr::DictComp(_) => None, // TODO: Handle dict comprehensions
        Expr::Generator(_) => todo!(),

        // Functions
        Expr::Call(_) => None,   // TODO: Handle call expressions
        Expr::Lambda(_) => None, // TODO: Handle lambda expressions
        Expr::Await(ExprAwait { value, .. }) => get_label_for_expression(checker, value), // Will go to the function expressions

        Expr::YieldFrom(_) | Expr::Yield(_) => None, // TODO: Handle yield expressions if needed

        Expr::FString(_) => todo!(),
        Expr::Starred(_) => todo!(), // TODO: Handle starred expressions

        Expr::IpyEscapeCommand(_) => None, // We dont support ipython escape commands (Jupyter)

        // Literals (are all public so return None)
        Expr::BooleanLiteral(_)
        | Expr::NumberLiteral(_)
        | Expr::StringLiteral(_)
        | Expr::NoneLiteral(_)
        | Expr::BytesLiteral(_)
        | Expr::EllipsisLiteral(_) => None,
    }
}

/// Check if a variable assignment has correct information flow, in terms of confidentiality.
/// I.e. the variable label is more restrictive than the value label or the same.
fn is_inconfidential_assign_statement(
    checker: &mut Checker,
    target: &Expr,
    value: &Expr,
) -> Option<IFInconfidentialVariableAssign> {
    // Get variable and value names
    let Expr::Name(variable_name) = target else {
        return None;
    };

    // TODO: Handle multiple targets

    // Get labels
    let variable_label = get_variable_label_by_name(checker, variable_name);
    let value_label = get_label_for_expression(checker, value);

    // No label for the variable or value, then it is not unauthorised
    if variable_label.is_none() || value_label.is_none() {
        return None;
    }

    if !variable_label
        .as_ref()
        .unwrap()
        .is_higher_in_lattice_path(value_label.as_ref().unwrap())
    {
        return Some(IFInconfidentialVariableAssign {
            var: variable_name.id.clone(),
            var_label: variable_label.unwrap(),
            expr: checker.locator().full_lines(value.range()).to_string(),
            expr_label: value_label.unwrap(),
        });
    } else {
        return None;
    }
}
