use std::collections::HashMap;

/* Common Traits */
trait Expr {
    fn exec_expr(&mut self, context: &Context) -> u64;
}

trait Stmt {
    fn exec_stmt(&mut self, context: &Context);
}

/* Print */
struct Print<T: Expr> {
    inner: T,
}

fn print<T: Expr>(inner: T) -> Print<T> {
    Print { inner }
}

impl<T: Expr> Stmt for Print<T> {
    fn exec_stmt(&mut self, context: &Context) {
        let value = self.inner.exec_expr(context);
        println!("{}", value);
    }
}

/* Nothing */
struct Nothing();

fn nothing() -> Nothing {
    Nothing()
}

impl Stmt for Nothing {
    fn exec_stmt(&mut self, _context: &Context) { /* nothing happens */
    }
}

/* Seq<T: Stmt, U: Stmt> */
struct Seq<T: Stmt, U: Stmt> {
    first: T,
    second: U,
}

fn seq<T: Stmt, U: Stmt>(first: T, second: U) -> Seq<T, U> {
    Seq { first, second }
}

impl<T: Stmt, U: Stmt> Stmt for Seq<T, U> {
    fn exec_stmt(&mut self, context: &Context) {
        self.first.exec_stmt(context);
        self.second.exec_stmt(context);
    }
}

impl<T: Stmt> Seq<T, Nothing> {
    fn shorten_1(self) -> T {
        self.first
    }
}

impl<T: Stmt> Seq<Nothing, T> {
    fn shorten_2(self) -> T {
        self.second
    }
}

impl Seq<Nothing, Nothing> {
    fn collapse(self) -> Nothing {
        Nothing()
    }
}

/* Expr for u64 */
impl Expr for u64 {
    fn exec_expr(&mut self, _context: &Context) -> u64 {
        *self
    }
}

/* When */
struct When<T: Expr, Y: Expr, U: Expr> {
    condition: T,
    if_true: Y,
    if_false: U,
}

fn when<T: Expr, Y: Expr, U: Expr>(condition: T, if_true: Y, if_false: U) -> When<T, Y, U> {
    When {
        condition,
        if_true,
        if_false,
    }
}

impl<T: Expr, Y: Expr, U: Expr> Expr for When<T, Y, U> {
    fn exec_expr(&mut self, context: &Context) -> u64 {
        let condition_value = self.condition.exec_expr(context);
        if condition_value == 0 {
            return self.if_false.exec_expr(context);
        }
        self.if_true.exec_expr(context)
    }
}

/* Repeat */
struct Repeat<const N: u32, T: Stmt> {
    inner: T,
}

fn repeat<const N: u32, T: Stmt>(inner: T) -> Repeat<N, T> {
    Repeat { inner }
}

impl<const N: u32, T: Stmt> Stmt for Repeat<N, T> {
    fn exec_stmt(&mut self, context: &Context) {
        for _ in 0..N {
            self.inner.exec_stmt(context);
        }
    }
}

/* Coexec_stmtnstant */
struct Constant {
    name: &'static str,
}

fn constant(name: &'static str) -> Constant {
    Constant { name }
}

impl Expr for Constant {
    fn exec_expr(&mut self, context: &Context) -> u64 {
        *context.get(self.name).unwrap() // TODO: revisit if the unwrap is good here
    }
}

/* ReadFrom<'a> */
struct ReadFrom<'a> {
    field: &'a u64,
}

fn read_from(field: &'_ u64) -> ReadFrom<'_> {
    ReadFrom { field }
}

impl<'a> Expr for ReadFrom<'a> {
    fn exec_expr(&mut self, _context: &Context) -> u64 {
        *self.field
    }
}

/* Save In */
struct SaveIn<'a, T: Expr> {
    destination: &'a mut u64,
    inner: T,
}

fn save_in<T: Expr>(destination: &mut u64, inner: T) -> SaveIn<'_, T> {
    SaveIn { destination, inner }
}

impl<T: Expr> Expr for SaveIn<'_, T> {
    fn exec_expr(&mut self, context: &Context) -> u64 {
        *self.destination = self.inner.exec_expr(context);
        *self.destination
    }
}

/* Volatile */
struct Volatile<'a, T: Expr> {
    destination: &'a mut u64,
    name: &'static str,
    inner: T,
}

fn volatile<'a, T: Expr>(
    destination: &'a mut u64,
    name: &'static str,
    inner: T,
) -> Volatile<'a, T> {
    Volatile {
        destination,
        name,
        inner,
    }
}

impl<'a, T: Expr> Expr for Volatile<'a, T> {
    fn exec_expr(&mut self, context: &Context) -> u64 {
        let mut new_context = context.clone();
        _ = new_context.insert(self.name, *self.destination);
        *self.destination = self.inner.exec_expr(&new_context);
        *self.destination
    }
}

type Context = HashMap<&'static str, u64>;

fn main() {
    let mut context: Context = Context::new();
    _ = context.insert("x", 10);
    _ = context.insert("y", 10);
    _ = context.insert("z", 10);

    let mut program =
        seq(
        print(when(constant("y"), 1, 0)),
        seq(nothing(), print(when(constant("x"), 2, 0))),
    );
    program.exec_stmt(&context);

    let seq1 = seq(nothing(), print(when(constant("y"), 4, 0)));
    seq1.shorten_2().exec_stmt(&context);
    let seq2 = seq(print(when(constant("x"), 8, 0)), nothing());
    seq2.shorten_1().exec_stmt(&context);
    let seqn = seq(nothing(), nothing());
    seqn.collapse().exec_stmt(&context);

    let mut rep = repeat::<5, _>(seq(nothing(), print(when(constant("z"), 16, 0))));
    rep.exec_stmt(&context);

    let mut x = 0;
    let mut rf = read_from(&x);
    rf.exec_expr(&context);


    let mut svin = save_in(&mut x, when(constant("z"), 32, 0));
    svin.exec_expr(&context);

    let mut vol = volatile(&mut x, "x", 64);
    vol.exec_expr(&context);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;

    // Ta struktura zapamiętuje `label` dla każdego wywałania siebie i tych,
    // którzy mają kopię `log`
    struct Recorder {
        label: &'static str,
        log: Rc<RefCell<Vec<&'static str>>>,
    }
    impl Stmt for Recorder {
        fn exec_stmt(&mut self, _context: &Context) {
            self.log.borrow_mut().push(self.label);
        }
    }

    // Ta struktura zlicza, ile razy ona i jej klony były wywołane
    struct CounterExpr {
        calls: Rc<RefCell<u32>>,
        value: u64,
    }
    impl Expr for CounterExpr {
        fn exec_expr(&mut self, _context: &Context) -> u64 {
            *self.calls.borrow_mut() += 1;
            self.value
        }
    }

    #[test]
    fn print_struct_executes_inner_once() {
        let ctx = HashMap::from([("x", 0), ("y", 0)]);
        let calls = Rc::new(RefCell::new(0u32));
        let ce = CounterExpr {
            calls: calls.clone(),
            value: 123,
        };
        let mut p = print(ce);
        p.exec_stmt(&ctx);
        assert_eq!(*calls.borrow(), 1);
    }

    #[test]
    fn nothing_struct_does_nothing() {
        let ctx = HashMap::from([("x", 0), ("y", 0)]);
        let mut n = nothing(); // TODO: changed from "let mut n = Nothing;". check if this was intentional
        n.exec_stmt(&ctx);
    }

    #[test]
    fn seq_struct_executes_in_order() {
        let ctx = HashMap::from([("x", 0), ("y", 0)]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let r1 = Recorder {
            label: "first",
            log: log.clone(),
        };
        let r2 = Recorder {
            label: "second",
            log: log.clone(),
        };
        let mut s = seq(r1, r2);
        s.exec_stmt(&ctx);
        assert_eq!(&*log.borrow(), &["first", "second"]);
    }

    #[test]
    fn seq_shorten_1_discards_trailing_nothing_and_returns_first() {
        let ctx = HashMap::from([("x", 0), ("y", 0)]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let r = Recorder {
            label: "A",
            log: log.clone(),
        };
        let s = seq(r, nothing());
        // shorten_1 should return the first statement (Recorder)
        let mut first_only = s.shorten_1();
        first_only.exec_stmt(&ctx);
        assert_eq!(&*log.borrow(), &["A"]);
    }

    #[test]
    fn seq_shorten_2_discards_leading_nothing_and_returns_second() {
        let ctx = HashMap::from([("x", 0), ("y", 0)]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let r = Recorder {
            label: "B",
            log: log.clone(),
        };
        let s = seq(nothing(), r);
        // shorten_2 should return the second statement (Recorder)
        let mut second_only = s.shorten_2();
        second_only.exec_stmt(&ctx);
        assert_eq!(&*log.borrow(), &["B"]);
    }

    #[test]
    fn seq_collapse_reduces_two_nothings_to_single_nothing() {
        let _collapsed: Nothing = seq(nothing(), nothing()).collapse();
    }

    #[test]
    fn when_struct_branches() {
        let ctx = HashMap::new();
        let mut expr0 = when(0, 7u64, 8u64);
        let mut expr1 = when(1, 7u64, 8u64);
        assert_eq!(expr0.exec_expr(&ctx), 8);
        assert_eq!(expr1.exec_expr(&ctx), 7);
    }

    #[test]
    fn repeat_struct_runs_n_times() {
        let ctx = HashMap::new();
        let log = Rc::new(RefCell::new(Vec::new()));
        let r = Recorder {
            label: "tick",
            log: log.clone(),
        };

        let mut rep = repeat::<3, _>(r);
        rep.exec_stmt(&ctx);
        assert_eq!(&*log.borrow(), &["tick", "tick", "tick"]);
    }

    #[test]
    fn constant_struct_reads_value() {
        let ctx = HashMap::from([("k", 123u64)]);
        let mut program = constant("k");
        assert_eq!(program.exec_expr(&ctx), 123);
    }

    #[test]
    fn readfrom_struct_returns_value() {
        let ctx = HashMap::new();
        let x: u64 = 99;
        let mut program = read_from(&x);
        assert_eq!(program.exec_expr(&ctx), 99);
    }

    #[test]
    fn savein_struct_writes_and_returns() {
        let ctx = HashMap::new();
        let mut dst: u64 = 0;
        let mut program = save_in(&mut dst, 123u64);
        let out = program.exec_expr(&ctx);
        assert_eq!(dst, 123);
        assert_eq!(out, 123);
    }

    #[test]
    fn volatile_struct_shadows_and_updates() {
        let ctx = HashMap::from([("y", 10)]);
        let mut a: u64 = 0;

        let mut v1 = volatile(&mut a, "y", when(constant("y"), 7u64, 8u64));
        let out1 = v1.exec_expr(&ctx);
        assert_eq!(out1, 8);
        assert_eq!(a, 8);

        let mut v2 = volatile(&mut a, "y", when(constant("y"), 7u64, 8u64));
        let out2 = v2.exec_expr(&ctx);
        assert_eq!(out2, 7);
        assert_eq!(a, 7);
    }

    // Nesting tests
    #[test]
    fn nesting_when_inside_when_structs() {
        let ctx1 = HashMap::from([("x", 1), ("y", 1)]);
        let ctx2 = HashMap::from([("x", 1), ("y", 0)]);
        let ctx3 = HashMap::from([("x", 0), ("y", 0)]);
        let mut nested = when(
            when(constant("y"), 1u64, 0u64),
            10u64,
            when(constant("x"), 20u64, 30u64),
        );
        assert_eq!(nested.exec_expr(&ctx1), 10);
        assert_eq!(nested.exec_expr(&ctx2), 20);
        assert_eq!(nested.exec_expr(&ctx3), 30);
    }

    #[test]
    fn nesting_seq_repeat_order_structs() {
        let ctx = HashMap::from([("x", 0), ("y", 0)]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let r_a = Recorder {
            label: "A",
            log: log.clone(),
        };
        let r_b = Recorder {
            label: "B",
            log: log.clone(),
        };
        let mut program = seq(repeat::<2, _>(r_a), repeat::<3, _>(r_b));
        program.exec_stmt(&ctx);
        assert_eq!(&*log.borrow(), &["A", "A", "B", "B", "B"]);
    }

    #[test]
    fn nesting_savein_then_volatile_structs() {
        let ctx = HashMap::from([("y", 0)]);
        let mut a: u64 = 0;
        let mut b: u64 = 0;
        let mut set_a = save_in(&mut a, 5u64);
        assert_eq!(set_a.exec_expr(&ctx), 5);
        let mut expr = save_in(
            &mut b,
            when(
                volatile(&mut a, "y", when(constant("y"), 1u64, 0u64)),
                9u64,
                10u64,
            ),
        );
        let out = expr.exec_expr(&ctx);
        assert_eq!(out, 9);
        assert_eq!(b, 9);
        assert_eq!(a, 1);
    }

    // Two integration tests that exercise everything
    #[test]
    fn integration_full_flow_1() {
        let ctx = HashMap::from([("x", 0), ("y", 10)]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut a: u64 = 0;
        let b: u64 = 0;

        // part1
        let mut part1 = seq(
            print(when(constant("y"), 1u64, 2u64)),
            print(when(constant("x"), 1u64, 2u64)),
        );
        part1.exec_stmt(&ctx);

        // part2: save into a, then read a in a separate step to avoid borrow conflicts
        let mut part2a = print(save_in(&mut a, when(constant("y"), 7u64, 8u64)));
        part2a.exec_stmt(&ctx);
        let mut part2b = print(read_from(&a));
        part2b.exec_stmt(&ctx);

        // part3
        let mut part3 = seq(
            repeat::<3, _>(Recorder {
                label: "tick",
                log: log.clone(),
            }),
            // Use `a` (currently 7) to shadow `y`, so branch -> 100
            print(volatile(&mut a, "y", when(constant("y"), 100u64, 200u64))),
        );
        part3.exec_stmt(&ctx);

        assert_eq!(a, 100);
        assert_eq!(b, 0);
        assert_eq!(&*log.borrow(), &["tick", "tick", "tick"]);
    }

    #[test]
    fn integration_full_flow_2() {
        let ctx = HashMap::from([("x", 1), ("y", 0)]);
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut a: u64 = 0;
        let mut b: u64 = 0;

        let mut a_set = save_in(&mut a, when(constant("x"), 9u64, 10u64));
        assert_eq!(a_set.exec_expr(&ctx), 9);
        let mut b_set = save_in(
            &mut b,
            when(
                volatile(&mut a, "y", when(constant("y"), 1u64, 0u64)),
                123u64,
                456u64,
            ),
        );
        assert_eq!(b_set.exec_expr(&ctx), 123);

        let mut program = seq(
            repeat::<2, _>(Recorder {
                label: "A",
                log: log.clone(),
            }),
            repeat::<1, _>(Recorder {
                label: "B",
                log: log.clone(),
            }),
        );
        program.exec_stmt(&ctx);

        assert_eq!(a, 1);
        assert_eq!(b, 123);
        assert_eq!(&*log.borrow(), &["A", "A", "B"]);
    }
}
