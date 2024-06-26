/// Basic rule checks
///
/// This module contains the rules that check for usage of the information flow rules, e.g. explicit labels for variables
/// <https://en.wikipedia.org/wiki/Information_flow_(information_theory) />

pub(crate) use must_include_variable_label::*;
mod must_include_variable_label;

pub(crate) use missing_principal::*;
mod missing_principal;
