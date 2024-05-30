use super::information_flow_state::InformationFlowState;
use super::label::Label;
use itertools::Itertools;
use ruff_python_ast::{
    Expr, ExprAttribute, ExprAwait, ExprBinOp, ExprBoolOp, ExprCall, ExprCompare, ExprDict, ExprIf,
    ExprList, ExprNamed, ExprSet, ExprSlice, ExprSubscript, ExprTuple, ExprUnaryOp,
};
use ruff_python_ast::{ExprName, Stmt};
use ruff_python_semantic::SemanticModel;

/// Fetch the label of a variable in the given scope
pub(crate) fn get_variable_label_by_name(
    semantic: &SemanticModel,
    information_flow: &InformationFlowState,
    name: &ExprName,
) -> Option<Label> {
    // Get shadowed [BindingId] from [Scope] if it exists. We only have to check shadowed bindings,
    // because otherwise the variable is new and does not have a label
    let binding_id = if let Some(binding_id) = semantic.current_scope().get(name.id.as_str()) {
        binding_id
    } else if let Some(binding_id) = semantic.resolve_name(name) {
        // check outer scopes for deferred functions
        binding_id
    } else {
        return None;
    };

    if let Some(scope) = semantic.get_binding_scope(binding_id) {
        return scope
            .shadowed_bindings(binding_id)
            .find_map(|bid| information_flow.get_label(bid));
    } else {
        None
    }
}

pub(crate) fn get_combination_of_labels_from_list_of_labels(labels: Vec<Option<Label>>) -> Label {
    let mut curr_label: Label = Label::new_public();
    for label in labels.into_iter().flatten() {
        curr_label += label;
    }

    curr_label
}

pub(crate) fn get_list_of_labels_labels(
    semantic: &SemanticModel,
    information_flow: &InformationFlowState,
    expressions: &Vec<Expr>,
) -> Vec<Option<Label>> {
    expressions
        .iter()
        .map(|expr| get_label_for_expression(semantic, information_flow, expr))
        .collect_vec()
}

pub(crate) fn get_label_from_statement(
    semantic: &SemanticModel,
    information_flow: &InformationFlowState,
    stmt: &Stmt,
) -> Option<Label> {
    if let Some(expr) = match stmt {
        Stmt::Assign(assign) => Some(&assign.value),
        Stmt::AnnAssign(ann_assign) => ann_assign.value.as_ref(),
        Stmt::AugAssign(aug_assign) => Some(&aug_assign.value),
        _ => None,
    } {
        get_label_for_expression(semantic, information_flow, expr)
    } else {
        None
    }
}

/// Get the label of an expression.
/// Returns the most restrictive label found in the expression
/// TODO: Return the range as well and name for a more detailed error message
pub(crate) fn get_label_for_expression(
    semantic: &SemanticModel,
    information_flow: &InformationFlowState,
    expr: &Expr,
) -> Option<Label> {
    match expr {
        Expr::Name(name) => get_variable_label_by_name(semantic, information_flow, name), // Get label from variable
        Expr::Subscript(ExprSubscript { slice, .. }) => {
            get_label_for_expression(semantic, information_flow, slice)
        }
        Expr::UnaryOp(ExprUnaryOp { operand, .. }) => {
            get_label_for_expression(semantic, information_flow, operand)
        }
        Expr::Named(ExprNamed {
            target: left,
            value: right,
            ..
        })
        | Expr::BinOp(ExprBinOp { left, right, .. }) => {
            let left_label = get_label_for_expression(semantic, information_flow, left);
            let right_label = get_label_for_expression(semantic, information_flow, right);

            Some(get_combination_of_labels_from_list_of_labels(vec![
                left_label,
                right_label,
            ]))
        }

        Expr::Slice(ExprSlice {
            lower, upper, step, ..
        }) => {
            let lower_label = if lower.is_none() {
                None
            } else {
                get_label_for_expression(semantic, information_flow, lower.as_ref().unwrap())
            };
            let upper_label = if upper.is_none() {
                None
            } else {
                get_label_for_expression(semantic, information_flow, upper.as_ref().unwrap())
            };
            let step_label = if step.is_none() {
                None
            } else {
                get_label_for_expression(semantic, information_flow, step.as_ref().unwrap())
            };

            Some(get_combination_of_labels_from_list_of_labels(vec![
                lower_label,
                upper_label,
                step_label,
            ]))
        }

        // Expressions with dynamic number of values (i.e. a vector of expressions)
        Expr::BoolOp(ExprBoolOp { values, .. })
        | Expr::Tuple(ExprTuple { elts: values, .. })
        | Expr::List(ExprList { elts: values, .. })
        | Expr::Set(ExprSet { elts: values, .. }) => {
            Some(get_combination_of_labels_from_list_of_labels(
                get_list_of_labels_labels(semantic, information_flow, values),
            ))
        }

        Expr::Dict(ExprDict { items, .. }) => {
            let value_labels = items
                .iter()
                .map(|item| item.value.clone())
                .map(|value| get_label_for_expression(semantic, information_flow, &value))
                .collect_vec();
            Some(get_combination_of_labels_from_list_of_labels(value_labels))
        }

        Expr::Attribute(ExprAttribute { value, .. }) => {
            // Get label from object
            get_label_for_expression(semantic, information_flow, value)
        } // TODO: For now we only handle the object label. Handle attribute individual attributes expressions (i.e. attributes from classes)

        Expr::Compare(ExprCompare {
            left, comparators, ..
        }) => {
            let left_label = get_label_for_expression(semantic, information_flow, left);
            let right_label = get_combination_of_labels_from_list_of_labels(
                comparators
                    .into_iter()
                    .map(|e| get_label_for_expression(semantic, information_flow, e))
                    .collect_vec(),
            );

            Some(get_combination_of_labels_from_list_of_labels(vec![
                left_label,
                Some(right_label),
            ]))
        }
        Expr::If(ExprIf {
            body, test, orelse, ..
        }) => {
            let test_label = get_label_for_expression(semantic, information_flow, test);
            let body_label = get_label_for_expression(semantic, information_flow, body);
            let orelse_label = get_label_for_expression(semantic, information_flow, orelse);

            Some(get_combination_of_labels_from_list_of_labels(vec![
                test_label,
                body_label,
                orelse_label,
            ]))
        }

        // Comprehensions
        Expr::ListComp(_) => None, // TODO: Handle list comprehensions
        Expr::SetComp(_) => None,  // TODO: Handle set comprehensions
        Expr::DictComp(_) => None, // TODO: Handle dict comprehensions
        Expr::Generator(_) => None,

        // Functions
        Expr::Call(ExprCall { func, .. }) => {
            get_label_for_expression(semantic, information_flow, func)
        }
        Expr::Lambda(_) => None, // TODO: Handle lambda expressions
        Expr::Await(ExprAwait { value, .. }) => {
            get_label_for_expression(semantic, information_flow, value)
        } // Will go to the function expressions

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
        | Expr::EllipsisLiteral(_) => Some(information_flow.default_label()), // Principle of least privilege TODO: add setting for this?
    }
}
