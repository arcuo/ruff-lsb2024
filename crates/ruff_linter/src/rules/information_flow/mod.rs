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
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_var.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_inherit.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_expr.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_fun.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_ann.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_aug.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_inline.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_list.py"))]
    #[test_case(Rule::IFExplicitVariableAssign, Path::new("IF101/IF101_mult.py"))]
    #[test_case(
        Rule::IFExplicitVariableAssign,
        Path::new("IF101/IF101_tuple_assign.py")
    )]
    // Implicit
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201/IF201_if.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201/IF201_while.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201/IF201_for.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201/IF201_nested.py"))]
    #[test_case(Rule::IFImplicitVariableAssign, Path::new("IF201/IF201_assert.py"))]
    #[test_case(Rule::IFImplicitFunctionReturn, Path::new("IF202.py"))]
    #[test_case(Rule::IFImplicitArgument, Path::new("IF203.py"))]
    #[test_case(Rule::IFImplicitFunctionReturn, Path::new("IF101/test.py"))]
    #[test_case(Rule::IFImplicitArgument, Path::new("IF101/test.py"))]

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
