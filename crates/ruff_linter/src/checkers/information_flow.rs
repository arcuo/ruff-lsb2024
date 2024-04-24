use std::{collections::VecDeque, str::FromStr};

use ruff_python_index::Indexer;
use ruff_python_semantic::BindingId;
use ruff_source_file::Locator;
use rustc_hash::FxHashMap;

/// State of the information flow
pub struct InformationFlowState<'a> {
    // The current principles of the program, e.g. ['alice', 'bob']
    principals: &'a Vec<String>,
    // The current scope level queue. The level is updated according to the scope by popping and
    pc: VecDeque<String>,
    // Map from variable name to
    variable_map: FxHashMap<BindingId, String>,
}

impl<'a> InformationFlowState<'a> {
    pub(crate) fn new(indexer: &'a Indexer, locator: &'a Locator) -> Self {
        Self {
            principals: &initiate_principals(indexer, locator),
            variable_map: FxHashMap::default(),
            pc: VecDeque::new(),
        }
    }

    /// Return the current level of the information flow state
    pub(crate) fn get_pc(&self) -> &String {
        return self.pc.front().unwrap_or_else(|| &String::default());
    }
}

#[derive(Debug, PartialEq)]
struct Principals<'a> {
    principals: &'a Vec<String>,
}

impl FromStr for Principals {
    /// Parses a string of principals e.g. from a comment
    /// ```
    /// ifprincipals {
    ///   alice,
    ///   bob
    /// }
    /// ```
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        string.trim()
    }
    
    type Err;
}

/// Initiate the principals list
fn initiate_principals(indexer: &Indexer, locator: &Locator) -> Vec<String> {
    let principals: Vec<String> = vec![];

    let bcomments = indexer.comment_ranges().block_comments(locator);
    if bcomments.len() == 0 {
        // Found no block comments and therefore no principals
        // TODO: Should we allow other representations? I.e. configurations
        return principals;
    } else {
    }

    return principals;
}
