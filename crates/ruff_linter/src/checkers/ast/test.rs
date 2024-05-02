use std::path::Path;

use ruff_python_index::Indexer;
use ruff_python_parser::{lexer::LexResult, Mode};
use ruff_python_semantic::Module;
use ruff_source_file::Locator;

use crate::{
    directives, linter::TokenSource, packaging::detect_package_root, settings::{flags::Noqa, LinterSettings}
};

use super::Checker;

// TODO: Should we use this?

/// Create a new [`Checker`] for the given source code
pub(crate) fn test_checker_state_after_snippet_traversal(source: &str) -> &Checker {
    let path = Path::new("");
    let locator = Locator::new(source);
    let tokens = TokenSource::Tokens(ruff_python_parser::tokenize_all(source, Mode::Module));
    let indexer = Indexer::from_tokens(&tokens, &locator);
    let settings = LinterSettings::default();
    let directives = directives::extract_directives(
        &tokens,
        directives::Flags::from_settings(&settings),
        &locator,
        &indexer,
    );
    let checker = Checker::new(
        &settings,
        &directives.noqa_line_for,
        Noqa::Disabled,
        &path,
        path.parent()
            .and_then(|parent| detect_package_root(parent, &settings.namespace_packages)),
        Module {
            kind: ruff_python_semantic::ModuleKind::Module,
            source: ruff_python_semantic::ModuleSource::Path(&[path.to_str().unwrap().to_string()]),
            python_ast: tokens.,
            name: path.to_string(),
        },
    );
    &checker
}
