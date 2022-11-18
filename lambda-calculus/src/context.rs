use crate::raw_expr::*;

#[derive(Clone, Copy, Debug)]
pub enum Context<'a> {
    Root,
    Var {
        param: &'a Param,
        parent: &'a Context<'a>,
    },
}

impl<'a> Context<'a> {
    pub fn get_var(&self, mut idx: DeBruijnIndex) -> &'a Param {
        /* The recursive version is much nicer, but Rust has no tail recursion guarantee? */
        let mut ctx = self;
        loop {
            match ctx {
                Context::Root => panic!("invalid De Bruijn index"),
                Context::Var { param, parent } => {
                    if idx == 0 {
                        return param;
                    } else {
                        ctx = parent;
                        idx -= 1;
                    }
                }
            }
        }
    }

    pub fn get_var_index(&self, name: &str) -> Option<DeBruijnIndex> {
        let mut ctx = self;
        let mut idx: DeBruijnIndex = 0;
        loop {
            match ctx {
                Context::Root => return None,
                Context::Var { param, parent } => {
                    if param.name == name {
                        return Some(idx);
                    } else {
                        ctx = parent;
                        idx += 1;
                    }
                }
            }
        }
    }
}

pub struct WithContext<'a, T> {
    pub context: Context<'a>,
    pub obj: T,
    pub parens_for_app: bool,
    pub parens_for_lambda: bool,
}

impl<'a, T> WithContext<'a, T> {
    pub fn root(obj: T) -> Self {
        WithContext {
            context: Context::Root,
            obj,
            parens_for_app: false,
            parens_for_lambda: false,
        }
    }

    fn propagate<T2>(&self, obj: T2) -> WithContext<'a, T2> {
        WithContext {
            context: self.context,
            obj,
            parens_for_app: self.parens_for_app,
            parens_for_lambda: self.parens_for_lambda,
        }
    }
}

impl<'a> WithContext<'a, DeBruijnIndex> {
    pub fn get_param(&self) -> &'a Param {
        self.context.get_var(self.obj)
    }
}

impl<'a> WithContext<'a, &RawAppExpr> {
    pub fn get_fun(&self) -> WithContext<'a, &RawExpr> {
        WithContext {
            context: self.context,
            obj: &self.obj.fun,
            parens_for_app: false,
            parens_for_lambda: true,
        }
    }

    pub fn get_arg(&self) -> WithContext<'a, &RawExpr> {
        WithContext {
            context: self.context,
            obj: &self.obj.arg,
            parens_for_app: true,
            parens_for_lambda: true,
        }
    }
}

impl<'a> WithContext<'a, &RawLambdaExpr> {
    pub fn get_body(&'a self) -> WithContext<'a, &RawExpr> {
        WithContext {
            context: Context::Var {
                param: &self.obj.param,
                parent: &self.context,
            },
            obj: &self.obj.body,
            parens_for_app: true,
            parens_for_lambda: false,
        }
    }
}

pub enum ContextExprInfo<'a> {
    Var(WithContext<'a, DeBruijnIndex>),
    App(WithContext<'a, &'a RawAppExpr>),
    Lambda(WithContext<'a, &'a RawLambdaExpr>),
}

impl<'a> WithContext<'a, &RawExpr> {
    pub fn info(&'a self) -> ContextExprInfo<'a> {
        match self.obj {
            RawExpr::Var(var) => ContextExprInfo::Var(self.propagate(*var)),
            RawExpr::App(app) => ContextExprInfo::App(self.propagate(app)),
            RawExpr::Lambda(lambda) => ContextExprInfo::Lambda(self.propagate(lambda)),
        }
    }
}
