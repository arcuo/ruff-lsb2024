use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCall, ExprCompare, ExprDict, ExprIf,
    ExprList, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};
use ruff_python_semantic::BindingId;

use crate::checkers::ast::Checker;

use super::label::Label;

use ruff_python_ast::ExprName;

/// Fetch the label of a variable in the given scope
pub(crate) fn get_variable_label_by_name(checker: &mut Checker, name: &ExprName) -> Option<Label> {
    // Get shadowed [BindingId] from [Scope] if it exists. We only have to check shadowed bindings,
    // because otherwise the variable is new and does not have a label
    if let Some(binding_id) = checker.semantic().current_scope().get(name.id.as_str()) {
        return checker
            .semantic()
            .current_scope()
            .shadowed_bindings(binding_id)
            .find_map(|bid| checker.information_flow().get_label(bid));
    }

    None
}

pub(crate) fn get_most_restrictive_label_from_list_of_expressions(
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

pub(crate) fn get_higher_of_two_labels(
    label1: Option<Label>,
    label2: Option<Label>,
) -> Option<Label> {
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
pub(crate) fn get_label_for_expression(checker: &mut Checker, expr: &Expr) -> Option<Label> {
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
        Expr::Generator(_) => None,

        // Functions
        Expr::Call(ExprCall { func, .. }) => get_label_for_expression(checker, func),
        Expr::Lambda(_) => None, // TODO: Handle lambda expressions
        Expr::Await(ExprAwait { value, .. }) => get_label_for_expression(checker, value), // Will go to the function expressions

        Expr::YieldFrom(_) | Expr::Yield(_) => None, // TODO: Handle yield expressions if needed

        Expr::FString(_) => None,
        Expr::Starred(_) => None, // TODO: Handle starred expressions

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
