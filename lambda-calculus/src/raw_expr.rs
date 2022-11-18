use std::{mem::take, rc::Rc};

pub type DeBruijnIndex = u32;

#[derive(Debug)]
pub struct Param {
    pub name: String,
}

#[derive(Clone)]
pub enum RawExpr {
    Var(DeBruijnIndex),
    App(Box<RawAppExpr>),
    Lambda(Box<RawLambdaExpr>),
}

impl RawExpr {
    /* Reduce the expression as much as possible, using at most the given number of steps.
     * Returns true if some reduction was performed. */
    pub fn reduce(&mut self, limit: &mut u32) -> bool {
        let mut reduced = false;
        while *limit > 0 {
            match self {
                RawExpr::Var(_) => {
                    break;
                }
                RawExpr::App(app) => {
                    if let Some(beta_red) = app.try_get_beta_reduced() {
                        *self = beta_red;
                        reduced = true;
                        *limit -= 1;
                    } else if app.fun.reduce(limit) || app.arg.reduce(limit) {
                        reduced = true;
                    } else {
                        break;
                    }
                }
                RawExpr::Lambda(lambda) => {
                    if let Some(eta_red) = lambda.try_get_eta_reduced() {
                        *self = eta_red;
                        reduced = true;
                        *limit -= 1;
                    } else if lambda.body.reduce(limit) {
                        reduced = true;
                    } else {
                        break;
                    }
                }
            }
        }
        reduced
    }

    /* Substitute the variable with the given De Bruijn index with the given expression, adjusting
     * indices as required. The value is assumed to live in the (idx + 1)th parent context of self.
     * The result is an expression where the specific De Bruijn index is eliminated. */
    fn substitute(&mut self, idx: DeBruijnIndex, value: &mut RawExpr, may_take_value: bool) {
        match self {
            RawExpr::Var(var) => {
                if *var == idx {
                    if may_take_value {
                        *self = take(value);
                        self.shift(0, idx);
                    } else {
                        *self = value.shifted(0, idx);
                    }
                } else if *var > idx {
                    *var -= 1;
                }
            }
            RawExpr::App(app) => app.substitute(idx, value, may_take_value),
            RawExpr::Lambda(lambda) => lambda.substitute(idx, value, may_take_value),
        }
    }

    /* If start is 0, clone this expression into a sub-context of the original context with count
     * binders in between.
     * If start is > 0, keep all lower De Bruijn indicies as-is, because they refer to parameters
     * that were cloned along with the expression. */
    fn shifted(&self, start: DeBruijnIndex, count: DeBruijnIndex) -> RawExpr {
        match self {
            RawExpr::Var(var) => RawExpr::Var(if *var >= start { var + count } else { *var }),
            RawExpr::App(app) => app.shifted(start, count).into(),
            RawExpr::Lambda(lambda) => lambda.shifted(start, count).into(),
        }
    }

    /* Mutating version of shifted. */
    fn shift(&mut self, start: DeBruijnIndex, count: DeBruijnIndex) {
        match self {
            RawExpr::Var(var) => {
                if *var >= start {
                    *var += count;
                }
            }
            RawExpr::App(app) => app.shift(start, count),
            RawExpr::Lambda(lambda) => lambda.shift(start, count),
        }
    }

    /* Reverse of shift: If the expression does not reference any of the variables in the given
     * range of De Bruijn indices, eliminite these indices and return true, otherwise do nothing and
     * return false. */
    fn try_unshift(&mut self, start: DeBruijnIndex, count: DeBruijnIndex) -> bool {
        match self {
            RawExpr::Var(var) => {
                if *var >= start + count {
                    *var -= count;
                    true
                } else {
                    *var < start
                }
            }
            RawExpr::App(app) => app.try_unshift(start, count),
            RawExpr::Lambda(lambda) => lambda.try_unshift(start, count),
        }
    }
}

impl Default for RawExpr {
    fn default() -> Self {
        RawExpr::Var(DeBruijnIndex::MAX)
    }
}

#[derive(Clone)]
pub struct RawAppExpr {
    pub fun: RawExpr,
    pub arg: RawExpr,
}

impl RawAppExpr {
    // Note: Invalidates self if and only if beta reduction is possible.
    fn try_get_beta_reduced(&mut self) -> Option<RawExpr> {
        if let RawExpr::Lambda(lambda) = &mut self.fun {
            let result = &mut lambda.body;
            result.substitute(0, &mut self.arg, true);
            return Some(take(result));
        }
        None
    }

    fn substitute(&mut self, idx: DeBruijnIndex, value: &mut RawExpr, may_take_value: bool) {
        // Optimization: If arg does not reference the variable, allow fun to take the value.
        if may_take_value && self.arg.try_unshift(idx, 1) {
            self.fun.substitute(idx, value, may_take_value);
            return;
        }

        self.fun.substitute(idx, value, false);
        self.arg.substitute(idx, value, may_take_value);
    }

    fn shifted(&self, start: DeBruijnIndex, count: DeBruijnIndex) -> RawAppExpr {
        RawAppExpr {
            fun: self.fun.shifted(start, count),
            arg: self.arg.shifted(start, count),
        }
    }

    fn shift(&mut self, start: DeBruijnIndex, count: DeBruijnIndex) {
        self.fun.shift(start, count);
        self.arg.shift(start, count);
    }

    fn try_unshift(&mut self, start: DeBruijnIndex, count: DeBruijnIndex) -> bool {
        if !self.fun.try_unshift(start, count) {
            return false;
        }
        if !self.arg.try_unshift(start, count) {
            self.fun.shift(start, count);
            return false;
        }
        true
    }
}

impl From<RawAppExpr> for RawExpr {
    fn from(expr: RawAppExpr) -> Self {
        RawExpr::App(Box::new(expr))
    }
}

#[derive(Clone)]
pub struct RawLambdaExpr {
    pub param: Rc<Param>,
    pub body: RawExpr,
}

impl RawLambdaExpr {
    // Note: Invalidates self if and only if eta reduction is possible.
    fn try_get_eta_reduced(&mut self) -> Option<RawExpr> {
        if let RawExpr::App(app) = &mut self.body {
            if let RawExpr::Var(0) = app.arg {
                let result = &mut app.fun;
                if result.try_unshift(0, 1) {
                    return Some(take(result));
                }
            }
        }
        None
    }

    fn substitute(&mut self, idx: DeBruijnIndex, value: &mut RawExpr, may_take_value: bool) {
        self.body.substitute(idx + 1, value, may_take_value);
    }

    fn shifted(&self, start: DeBruijnIndex, count: DeBruijnIndex) -> RawLambdaExpr {
        RawLambdaExpr {
            param: self.param.clone(),
            body: self.body.shifted(start + 1, count),
        }
    }

    fn shift(&mut self, start: DeBruijnIndex, count: DeBruijnIndex) {
        self.body.shift(start + 1, count);
    }

    fn try_unshift(&mut self, start: DeBruijnIndex, count: DeBruijnIndex) -> bool {
        self.body.try_unshift(start + 1, count)
    }
}

impl From<RawLambdaExpr> for RawExpr {
    fn from(expr: RawLambdaExpr) -> Self {
        RawExpr::Lambda(Box::new(expr))
    }
}
