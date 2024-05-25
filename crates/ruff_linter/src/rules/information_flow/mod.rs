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

    // #[test_case(Rule::IFMustIncludeVariableLabel, Path::new("IF001.py"))]
    // #[test_case(Rule::IFMissingPrincipal, Path::new("IF002.py"))]
    // #[test_case(Rule::IFInconfidentialVariableAssign, Path::new("IF101.py"))]
    #[test_case(Rule::IFInconfidentialVariableAssign, Path::new("IF101Function.py"))]

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
