fn main() {
    /* Var */
    let vx = Var::X;
    let vy = Var::Z;
    let vz = Var::Z;

    println!("var X = {:?}, var Y = {:?}, var Z = {:?}", vx.to_string(), vy.to_string(), vz.to_string());

    /* Const */
    let cnumeric = Const::Numeric(0);
    let cnamed = Const::Named("a".into());

    println!("constant numeric = {:?}, named numeric = {:?}", cnumeric.to_string(), cnamed.to_string());

    /* Expression (E) */
    let eadd = E::add(E::constant(Const::Numeric(1)), E::constant(Const::Numeric(1)));
    let eneg = E::neg(E::constant(Const::Numeric(1)));
    let emul = E::mul(E::constant(Const::Numeric(1)), E::constant(Const::Numeric(1)));
    let einv = E::inv(E::constant(Const::Numeric(2)));
    let econst = E::constant(Const::Named("c".into()));
    let efunc = E::func("f".into(), E::var(Var::X));
    let evar = E::var(Var::X);

    println!("(E) add = {}", eadd.to_string());
    println!("(E) neg = {}", eneg.to_string());
    println!("(E) mul = {}", emul.to_string());
    println!("(E) inv = {}", einv.to_string());
    println!("(E) econst = {}", econst.to_string());
    println!("(E) efunc = {}", efunc.to_string());
    println!("(E) evar = {}", evar.to_string());
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum Var {
    X,
    Y,
    Z,
}

impl Var {
    fn to_string(&self) -> String {
        match &self {
            Var::X => str::to_string("X"),
            Var::Y => str::to_string("Y"),
            Var::Z => str::to_string("Z"),
        }
    }
}

#[derive(Clone, Debug)]
enum Const {
    Numeric(i64),
    Named(String),
}

impl Const {
    fn to_string(&self) -> String {
        match &self {
            Const::Numeric(n) => n.to_string(),
            Const::Named(n) => n.clone(),
        }
    }
}

#[derive(Clone, Debug)]
enum E {
    Add(Box<E>, Box<E>),
    Neg(Box<E>),
    Mul(Box<E>, Box<E>),
    Inv(Box<E>),
    Const(Const),
    Func { name: String, arg: Box<E> },
    Var(Var),
}

impl E {
    fn add(e1: Box<Self>, e2: Box<Self>) -> Box<Self> {
        Box::new(E::Add(e1, e2))
    }

    fn neg(e: Box<Self>) -> Box<Self> {
        Box::new(E::Neg(e))
    }

    fn mul(e1: Box<Self>, e2: Box<Self>) -> Box<Self> {
        Box::new(E::Mul(e1, e2))
    }

    fn inv(e: Box<Self>) -> Box<Self> {
        Box::new(E::Inv(e))
    }

    fn constant(c: Const) -> Box<Self> {
        Box::new(E::Const(c))
    }

    fn func(name: String, arg: Box<Self>) -> Box<Self> {
        Box::new(E::Func { name, arg })
    }

    fn var(v: Var) -> Box<Self> {
        Box::new(E::Var(v))
    }

    fn to_string(&self) -> String {
        match self {
            E::Add(e1, e2) => format!("({} + {})", e1.to_string(), e2.to_string()),
            E::Neg(e) => format!("-({})", e.to_string()),
            E::Mul(e1, e2) => format!("({} * {})", e1.to_string(), e2.to_string()),
            E::Inv(e) => format!("1/({})", e.to_string()),
            E::Const(c) => c.to_string(),
            E::Func { name, arg } => format!("{}({})", name.to_string(), arg.to_string()),
            E::Var(v) => v.to_string(),
        }
    }

    fn arg_count(&self) -> u32 {
        match self {
            E::Const(_) | E::Var(_) => 0,
            E::Neg(e) | E::Func { name: _, arg: e } | E::Inv(e) => 1,
            E::Add(e1, e2) | E::Mul(e1, e2) => 2,
        }
    }

    fn diff(self, by: Var) -> Box<Self> {
        match (self) {
            E::Add(e1, e2) => E::add(e1.diff(by), e2.diff(by)),
            E::Neg(e) => E::neg(e.diff(by)),
            E::Mul(e1, e2) => E::add(
                E::mul(e1.clone().diff(by), e2.clone()),
                E::mul(e1, e2.diff(by)),
            ),
            E::Inv(e) => E::mul(
                E::neg(E::inv(E::mul(e.clone(), e.clone()))), /* -1 / f(x)^2 */
                e.diff(by), /* f'(x) */
            ),
            E::Const(c) => E::constant(Const::Numeric(0)),
            E::Func { name, arg } => E::mul(
                E::func(format!("{}_{}", name, by.to_string()), arg.clone()),
                arg.diff(by),
            ), //E::func(format!("({}_{})", name, by.to_string()), arg),
            E::Var(v) => {
                if v == by {
                    E::constant(Const::Numeric(1))
                } else {
                    E::constant(Const::Numeric(0))
                }
            }
        }
    }

    fn unpack_inv_inv(self) -> Option<Box<Self>> {
        if let E::Inv(e) = self {
            if let E::Inv(inner) = *e {
                return Some(inner);
            }
            return None;
        }
        None
    }

    fn uninv(self: Box<Self>) -> Box<Self> {
        let mut temp = self;
        while let Some(uninved) = temp.clone().unpack_inv_inv() {
            temp = uninved;
        }
        temp
    }

    fn unpack_neg_neg(self) -> Option<Box<Self>> {
        if let E::Neg(e) = self
            && let E::Neg(inner) = *e
        {
            return Some(inner);
        }
        None
    }

    fn unneg(self: Box<Self>) -> Box<Self> {
        let mut temp = self;
        while let Some(unneged) = temp.clone().unpack_neg_neg() {
            temp = unneged;
        }
        temp
    }

    fn substitute(self, name: &str, value: Box<Self>) -> Box<Self> {
        match &self {
            E::Add(e1, e2) => E::add(
                e1.clone().substitute(name, value.clone()),
                e2.clone().substitute(name, value),
            ),
            E::Neg(e) => E::neg(e.clone().substitute(name, value)),
            E::Mul(e1, e2) => E::mul(
                e1.clone().substitute(name, value.clone()),
                e2.clone().substitute(name, value),
            ),
            E::Inv(e) => E::inv(e.clone().substitute(name, value)),
            E::Const(c) => {
                if let Const::Named(n) = c {
                    if n == name { value } else { Box::new(self) }
                } else {
                    Box::new(self)
                }
            }
            E::Func { name: n, arg } => E::func(n.clone(), arg.clone().substitute(name, value)),
            E::Var(_) => Box::new(self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_const_to_string() {
        let c_num = Const::Numeric(42);
        let c_name = Const::Named("a".into());
        assert_eq!(c_num.to_string(), "42");
        assert_eq!(c_name.to_string(), "a");
    }

    #[test]
    fn test_var_to_string() {
        assert_eq!(Var::X.to_string(), "X");
        assert_eq!(Var::Y.to_string(), "Y");
        assert_eq!(Var::Z.to_string(), "Z");
    }

    #[test]
    fn test_builder_constant_var() {
        let e_const = E::constant(Const::Numeric(5));
        let e_var = E::var(Var::X);
        assert_eq!(e_const.to_string(), "5");
        assert_eq!(e_var.to_string(), "X");
    }

    #[test]
    fn test_builder_add() {
        let expr = E::add(E::constant(Const::Numeric(2)), E::var(Var::X));
        assert_eq!(expr.to_string(), "(2 + X)");
    }

    #[test]
    fn test_builder_neg() {
        let expr = E::neg(E::var(Var::X));
        assert_eq!(expr.to_string(), "-(X)");
    }

    #[test]
    fn test_builder_mul() {
        let expr = E::mul(E::var(Var::X), E::var(Var::Y));
        assert_eq!(expr.to_string(), "(X * Y)");
    }

    #[test]
    fn test_builder_inv() {
        let expr = E::inv(E::var(Var::X));
        assert_eq!(expr.to_string(), "1/(X)");
    }

    #[test]
    fn test_builder_func() {
        let expr = E::func("f".into(), E::var(Var::X));
        assert_eq!(expr.to_string(), "f(X)");
    }

    #[test]
    fn test_expr_to_string_complex() {
        let expr1 = E::add(E::constant(Const::Numeric(2)), E::var(Var::X));
        let expr2 = E::mul(E::neg(E::var(Var::Y)), E::inv(E::var(Var::Z)));
        let complex = E::add(
            E::func("f".into(), expr1.clone()),
            E::func("g".into(), expr2.clone()),
        );
        assert_eq!(complex.to_string(), "(f((2 + X)) + g((-(Y) * 1/(Z))))");
    }

    #[test]
    fn test_diff_add_vars() {
        let expr = E::add(E::var(Var::X), E::var(Var::Y));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "(1 + 0)");
    }

    #[test]
    fn test_unpack_inv_inv() {
        let double_inv = E::inv(E::inv(E::var(Var::X)));
        let inner = double_inv.clone().unpack_inv_inv().unwrap();
        assert_eq!(inner.to_string(), "X");
    }

    #[test]
    fn test_unpack_neg_neg() {
        let double_neg = E::neg(E::neg(E::neg(E::neg(E::neg(E::var(Var::Y))))));
        let inner = double_neg.clone().unneg();
        assert_eq!(inner.to_string(), "-(Y)");
    }

    #[test]
    fn test_simplify_double_inv() {
        let double_inv = E::inv(E::inv(E::var(Var::X)));
        let simplified = double_inv.uninv();
        assert_eq!(simplified.to_string(), "X");
    }

    #[test]
    fn test_simplify_double_neg() {
        let double_neg = E::neg(E::neg(E::var(Var::X)));
        let simplified = double_neg.unneg();
        assert_eq!(simplified.to_string(), "X");
    }

    #[test]
    fn test_substitute_named_constant() {
        let expr = E::add(E::constant(Const::Named("a".into())), E::var(Var::X));
        let substituted = expr.substitute("a", E::constant(Const::Numeric(10)));
        assert_eq!(substituted.to_string(), "(10 + X)");
    }

    #[test]
    fn test_substitute_deep() {
        let expr = E::mul(
            E::constant(Const::Named("a".into())),
            E::func("f".into(), E::constant(Const::Named("a".into()))),
        );
        let substituted = expr.substitute("a", E::constant(Const::Numeric(3)));
        assert_eq!(substituted.to_string(), "(3 * f(3))");
    }

    #[test]
    fn test_diff_neg() {
        let expr = E::neg(E::var(Var::X));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "-(1)");
    }

    #[test]
    fn test_diff_mul() {
        let expr = E::mul(E::var(Var::X), E::var(Var::Y));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "((1 * Y) + (X * 0))");
    }

    #[test]
    fn test_diff_inv() {
        let expr = E::inv(E::var(Var::X));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "(-(1/((X * X))) * 1)");
    }

    #[test]
    fn test_diff_const_numeric() {
        let expr = E::constant(Const::Numeric(7));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "0");
    }

    #[test]
    fn test_diff_const_named() {
        let expr = E::constant(Const::Named("a".into()));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "0");
    }

    #[test]
    fn test_diff_func() {
        let expr = E::func("f".into(), E::var(Var::X));
        let d = expr.diff(Var::X);
        assert_eq!(d.to_string(), "(f_X(X) * 1)");
    }

    #[test]
    fn test_diff_var_same() {
        let d = E::var(Var::X).diff(Var::X);
        assert_eq!(d.to_string(), "1");
    }

    #[test]
    fn test_diff_var_other() {
        let d = E::var(Var::Y).diff(Var::X);
        assert_eq!(d.to_string(), "0");
    }

    #[test]
    fn test_diff_big_expression() {
        // (((X + -(Y)) * 1/(Z)) + (f((X * Y)) + g(1/(X))))
        let part1 = E::add(E::var(Var::X), E::neg(E::var(Var::Y)));
        let part2 = E::inv(E::var(Var::Z));
        let a = E::mul(part1.clone(), part2.clone());
        let xy = E::mul(E::var(Var::X), E::var(Var::Y));
        let b = E::func("f".into(), xy);
        let inv_x = E::inv(E::var(Var::X));
        let c = E::func("g".into(), inv_x);
        let big = E::add(a.clone(), E::add(b.clone(), c.clone()));

        assert_eq!(
            big.to_string(),
            "(((X + -(Y)) * 1/(Z)) + (f((X * Y)) + g(1/(X))))"
        );

        let d = big.diff(Var::X);
        assert_eq!(
            d.to_string(),
            "((((1 + -(0)) * 1/(Z)) + ((X + -(Y)) * (-(1/((Z * Z))) * 0))) + ((f_X((X * Y)) * ((1 * Y) + (X * 0))) + (g_X(1/(X)) * (-(1/((X * X))) * 1))))"
        );
    }

    #[test]
    fn test_arg_count_zeroary() {
        assert_eq!(E::constant(Const::Numeric(1)).arg_count(), 0);
        assert_eq!(E::var(Var::X).arg_count(), 0);
    }

    #[test]
    fn test_arg_count_unary() {
        assert_eq!(E::neg(E::var(Var::X)).arg_count(), 1);
        assert_eq!(E::inv(E::var(Var::X)).arg_count(), 1);
        assert_eq!(E::func("f".into(), E::var(Var::X)).arg_count(), 1);
    }

    #[test]
    fn test_arg_count_binary() {
        assert_eq!(E::add(E::var(Var::X), E::var(Var::Y)).arg_count(), 2);
        assert_eq!(E::mul(E::var(Var::X), E::var(Var::Z)).arg_count(), 2);
    }
}
