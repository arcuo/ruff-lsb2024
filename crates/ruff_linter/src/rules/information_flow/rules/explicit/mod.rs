/// Explicit rule checks
/// 
/// This module contains the rules that check for explicit information flow.
/// https://en.wikipedia.org/wiki/Information_flow_(information_theory)#Explicit_flows_and_side_channels

use super::helpers;
pub(crate) use explicit_variable_assignment::*;
mod explicit_variable_assignment;
