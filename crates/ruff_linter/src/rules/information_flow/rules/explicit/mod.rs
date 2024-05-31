/// Explicit rule checks
///
/// This module contains the rules that check for explicit information flow.
/// <https://en.wikipedia.org/wiki/Information_flow_(information_theory)#Explicit_flows_and_side_channels />
pub(crate) use explicit_variable_assignment::*;
pub(crate) use explicit_function_return::*;
pub(crate) use explicit_arg_value::*;

mod explicit_variable_assignment;
mod explicit_function_return;
mod explicit_arg_value;
