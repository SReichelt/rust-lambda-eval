use std::fmt;

use crate::{context::*, raw_expr::*};

impl<'a> fmt::Display for WithContext<'a, DeBruijnIndex> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.get_param().name)
    }
}

impl<'a> fmt::Display for WithContext<'a, &RawAppExpr> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fun = self.get_fun();
        let arg = self.get_arg();
        if self.parens_for_app {
            write!(f, "({fun} {arg})")
        } else {
            write!(f, "{fun} {arg}")
        }
    }
}

impl<'a> fmt::Display for WithContext<'a, &RawLambdaExpr> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let param = &self.obj.param.name;
        let body = self.get_body();
        if self.parens_for_lambda {
            write!(f, "(λ{param}.{body})")
        } else {
            write!(f, "λ{param}.{body}")
        }
    }
}

impl<'a> fmt::Display for WithContext<'a, &RawExpr> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.info() {
            ContextExprInfo::Var(var) => var.fmt(f),
            ContextExprInfo::App(app) => app.fmt(f),
            ContextExprInfo::Lambda(lambda) => lambda.fmt(f),
        }
    }
}
