//! Settings for the `information_flow` plugin.

use crate::display_settings;
use ruff_macros::CacheKey;
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Default, Debug, Eq, PartialEq, Clone, CacheKey, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SecurityProperty {
    #[default]
    Confidentiality,
    Integrity,
    Both,
}

impl fmt::Display for SecurityProperty {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityProperty::Confidentiality => write!(f, "confidentiality"),
            SecurityProperty::Integrity => write!(f, "integrity"),
            SecurityProperty::Both => write!(f, "confidentiality and integrity"),
        }
    }
}

impl SecurityProperty {
    /// Returns `true` if this is an [`SecurityProperty::Confidentiality`].
    pub const fn is_confidentiality(&self) -> bool {
        matches!(self, SecurityProperty::Confidentiality)
    }

    /// Returns `true` if this is an [`SecurityProperty::Integrity`].
    pub const fn is_integrity(&self) -> bool {
        matches!(self, SecurityProperty::Integrity)
    }

    /// Returns `true` if this is an [`SecurityProperty::Both`].
    pub const fn is_both(&self) -> bool {
        matches!(self, SecurityProperty::Both)
    }

    /// Returns `true` if self and other don't match and self isn't [`SecurityProperty::Both`].
    pub fn skip_diagnostic(&self, other: &SecurityProperty) -> bool {
        if other.is_both() || self.is_both() {
            return false;
        }
        self != other
    }

    pub fn get_description(
        &self,
        target: &String,
        target_label: String,
        value: &String,
        value_label: String,
    ) -> String {
        match self {
            SecurityProperty::Confidentiality => {
                format!("{}@{} < {}@{}", target, target_label, value, value_label)
            }
            SecurityProperty::Integrity => {
                format!("{}@{} > {}@{}", target, target_label, value, value_label)
            }
            SecurityProperty::Both => {
                format!(
                    "{}@{} != {}@{}",
                    target, target_label, value, value_label
                )
            }
        }
    }

    pub fn get_description_pc(
        &self,
        target: &String,
        target_label: String,
        pc_label: String,
    ) -> String {
        match self {
            SecurityProperty::Confidentiality => {
                format!("{}@{} < pc@{}", target, target_label, pc_label)
            }
            SecurityProperty::Integrity => {
                format!("{}@{} > pc@{}", target, target_label, pc_label)
            }
            SecurityProperty::Both => {
                format!(
                    "{}@{} != pc@{}",
                    target, target_label, pc_label
                )
            }
        }
    }

    pub fn get_description_arg(
        &self,
        argname: &String,
        arg_label: String,
        defined_arg_label: String,
    ) -> String {
        match self {
            SecurityProperty::Confidentiality => {
                format!("{}@{} > {}", argname, defined_arg_label, arg_label)
            }
            SecurityProperty::Integrity => {
                format!("{}@{} < {}", argname, defined_arg_label, arg_label)
            }
            SecurityProperty::Both => {
                format!(
                    "{}@{} != {}",
                    argname, defined_arg_label, arg_label
                )
            }
        }
    }

    pub fn get_description_return(
        &self,
        return_expr: &String,
        return_label: String,
        defined_return_label: String,
    ) -> String {
        match self {
            SecurityProperty::Confidentiality => {
                format!(
                    "{}@{} > {}",
                    return_expr, return_label, defined_return_label
                )
            }
            SecurityProperty::Integrity => {
                format!(
                    "{}@{} < {}",
                    return_expr, return_label, defined_return_label
                )
            }
            SecurityProperty::Both => {
                format!(
                    "{}@{} != {}",
                    return_expr, return_label, defined_return_label
                )
            }
        }
    }
}

#[derive(Debug, Clone, Default, CacheKey)]
pub struct Settings {
    pub security_property: SecurityProperty,
}

impl fmt::Display for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        display_settings! {
            formatter = f,
            namespace = "linter.information_flow",
            fields = [
                self.security_property
            ]
        }
        Ok(())
    }
}
