use std::rc::Rc;

use crate::{context::*, raw_expr::*};

pub struct ParserInput<'a> {
    pub s: &'a str,
}

impl<'a> ParserInput<'a> {
    fn skip_whitespace(&mut self) {
        self.s = self.s.trim_start();
    }

    fn try_read_char(&mut self, c: char) -> bool {
        if let Some(rest) = self.s.strip_prefix(c) {
            self.s = rest;
            true
        } else {
            false
        }
    }

    fn read_char(&mut self, c: char) -> Result<(), String> {
        if self.try_read_char(c) {
            Ok(())
        } else {
            let rest = self.s;
            Err(format!("Expected {c} instead of: {rest}"))
        }
    }

    fn try_read_name(&mut self) -> Option<&str> {
        let s = self.s;
        let end = s
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(s.len());
        if end == 0 {
            None
        } else {
            self.s = &s[end..];
            Some(&s[..end])
        }
    }
}

impl RawExpr {
    pub fn parse(input: &mut ParserInput, context: &Context) -> Result<Self, String> {
        if let Some(mut expr) = Self::try_parse_one(input, context)? {
            while let Some(arg) = Self::try_parse_one(input, context)? {
                expr = RawAppExpr { fun: expr, arg }.into();
            }
            Ok(expr)
        } else {
            let rest = input.s;
            Err(format!("Expected expression instead of: {rest}"))
        }
    }

    fn try_parse_one(input: &mut ParserInput, context: &Context) -> Result<Option<Self>, String> {
        input.skip_whitespace();
        if input.try_read_char('(') {
            let expr = Self::parse(input, context)?;
            input.skip_whitespace();
            input.read_char(')')?;
            Ok(Some(expr))
        } else if input.try_read_char('λ') || input.try_read_char('\\') {
            input.skip_whitespace();
            let binder = Self::try_parse_binder(input, context)?;
            if binder.is_some() {
                Ok(binder)
            } else {
                let rest = input.s;
                Err(format!("Expected variable name instead of: {rest}"))
            }
        } else if let Some(name) = input.try_read_name() {
            if let Some(idx) = context.get_var_index(name) {
                Ok(Some(RawExpr::Var(idx)))
            } else {
                Err(format!("Variable {name} not found."))
            }
        } else {
            Ok(None)
        }
    }

    fn try_parse_binder(
        input: &mut ParserInput,
        context: &Context,
    ) -> Result<Option<Self>, String> {
        if let Some(name) = input.try_read_name() {
            let param = Rc::new(Param { name: name.into() });
            input.skip_whitespace();
            let body_context = Context::Var {
                param: &param,
                parent: context,
            };
            if let Some(body) = Self::try_parse_binder(input, &body_context)? {
                Ok(Some(RawLambdaExpr { param, body }.into()))
            } else {
                input.read_char('.')?;
                let body = Self::parse(input, &body_context)?;
                Ok(Some(RawLambdaExpr { param, body }.into()))
            }
        } else {
            Ok(None)
        }
    }
}
