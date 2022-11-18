use core::fmt;
use std::str::FromStr;

use crate::{context::*, parse::*, raw_expr::*};

// Allow raw expressions to be printed directly.
// Warning: will panic if an expression is not closed.
impl fmt::Display for RawExpr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        WithContext::root(self).fmt(f)
    }
}

impl FromStr for RawExpr {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut input = ParserInput { s };
        let expr = RawExpr::parse(&mut input, &Context::Root)?;
        let rest = input.s;
        if !rest.is_empty() {
            return Err(format!(
                "Expected expression or end of input instead of: {rest}"
            ));
        }
        Ok(expr)
    }
}
