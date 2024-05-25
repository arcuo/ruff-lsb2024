//! Rules for information flow.
pub(crate) mod rules;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Result;
    use test_case::test_case;

    use crate::registry::Rule;
    use crate::test::test_path;
    use crate::{assert_messages, settings};

    // Basic rules
    #[test_case(Rule::IFMustIncludeVariableLabel, Path::new("IF001.py"))]
    #[test_case(Rule::IFMissingPrincipal, Path::new("IF002.py"))]
    // Explicit
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101.py"))] // TODO: split into multiple tests
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101_for.py"))] // TODO: split into multiple tests

    // Implicit
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201_if.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201_while.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201_for.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201_nested.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201_assert.py"))]

    fn rules(rule_code: Rule, path: &Path) -> Result<()> {
        let snapshot = format!("{}_{}", rule_code.noqa_code(), path.to_string_lossy());
        let diagnostics = test_path(
            Path::new("information_flow").join(path).as_path(),
            &settings::LinterSettings::for_rule(rule_code),
        )?;
        assert_messages!(snapshot, diagnostics);
        Ok(())
    }
}
