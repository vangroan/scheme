use std::cell::{Ref, RefCell};
use std::fmt;
use std::fmt::Formatter;
use std::rc::Rc;

use smol_str::SmolStr;

use crate::env::Env;
use crate::error::Result;
use crate::handle::{Handle, RcWeak};
use crate::opcode::Op;


/// Shorthand utilities.
pub mod utils {
    use super::*;

    pub fn nil() -> Expr {
        Expr::Nil
    }

    pub fn cons(car: impl Into<Expr>, cdr: impl Into<Expr>) -> Expr {
        Expr::Pair(Handle::new(Pair(car.into(), cdr.into())))
    }
}

#[derive(Debug, Clone)]
pub struct Pair(pub(crate) Expr, pub(crate) Expr);

impl Pair {
    pub const fn new(car: Expr, cdr: Expr) -> Self {
        Pair(car, cdr)
    }

    #[inline]
    pub fn to_expr(self) -> Expr {
        Expr::Pair(Handle::new(self))
    }

    pub const fn split_first(&self) -> (&Expr, &Expr) {
        (&self.0, &self.1)
    }

    pub const fn head(&self) -> &Expr {
        &self.0
    }

    pub const fn rest(&self) -> &Expr {
        &self.1
    }

    pub fn set_head(&mut self, value: Expr) {
        self.0 = value;
    }

    pub fn set_tail(&mut self, value: Expr) {
        self.1 = value;
    }

    pub fn new_list(elements: &[Expr]) -> Option<Pair> {
        match elements.split_first() {
            Some((first, rest)) => {
                let head = first.clone();
                let tail: Expr = Pair::new_list(rest)
                    .map(|pair| Expr::Pair(Handle::new(pair)))
                    .unwrap_or(Expr::Nil);
                Some(Pair(head, tail))
            }
            None => None,
        }
    }

    /// Create a new well-formed list by taking ownership of the given elements.
    pub fn new_list_vec(mut elements: Vec<Expr>) -> Option<Pair> {
        // It's more performant to pop elements off the back of a vector.
        elements.reverse();
        Pair::new_list_vec_recursive(elements.pop(), elements)
    }

    fn new_list_vec_recursive(
        maybe_head: Option<Expr>,
        mut rest_reversed: Vec<Expr>,
    ) -> Option<Pair> {
        match maybe_head {
            Some(head) => Some(Pair(
                head,
                Pair::new_list_vec_recursive(rest_reversed.pop(), rest_reversed)
                    .map(Pair::to_expr)
                    .unwrap_or(Expr::Nil),
            )),
            None => None,
        }
    }

    pub(crate) fn is_list(expr: &Expr) -> bool {
        match expr {
            Expr::Pair(pair_handle) => {
                let pair = pair_handle.borrow();
                let rest = pair.rest();
                match rest {
                    Expr::Nil => true,
                    _ => Pair::is_list(rest),
                }
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Expr {
    /// Nil, null or none.
    ///
    /// While Lisp may have gotten an official `nil` value, Scheme settled
    /// on using an empty quoted s-expression. We implicitly detect this case
    /// and convert it to this special [`Nil`] value.
    ///
    /// ```scheme
    /// '()
    /// ```
    Nil,
    /// Returned by special forms or procedures that only have side-effects,
    /// but don't evaluate to values.
    ///
    /// Examples are `define`, `display` and `newline`.
    ///
    /// Also the value of a variable that was declared, but never defined.
    Void,
    Bool(bool),
    Number(f64),
    String(String),
    Ident(SmolStr),
    Keyword(Keyword),
    Quote(Box<Expr>),
    // TODO: List must be a linked list
    List(Vec<Expr>),
    // TODO: Handle of tuples, or tuple of handles?
    Pair(Handle<Pair>),
    Vector(Vec<Expr>),
    Sequence(Vec<Expr>),
    Procedure(Rc<Proc>),
    Closure(Handle<Closure>),
    NativeFunc(NativeFunc),
}

impl Expr {
    pub fn is_boolean(&self) -> bool {
        matches!(self, Expr::Bool(_))
    }

    pub fn as_number(&self) -> Option<f64> {
        match self {
            Expr::Number(number) => Some(*number),
            _ => None,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self, Expr::Number(_))
    }

    pub fn as_slice(&self) -> Option<&[Expr]> {
        match self {
            Expr::List(list) => Some(list.as_slice()),
            Expr::Sequence(sequence) => Some(sequence.as_slice()),
            _ => None,
        }
    }

    pub fn as_ident(&self) -> Option<&str> {
        match self {
            Expr::Ident(name) => Some(name.as_str()),
            _ => None,
        }
    }

    pub fn as_pair(&self) -> Ref<Pair> {
        match self {
            Expr::Pair(pair_handle) => pair_handle.borrow(),
            _ => panic!("expression is not a pair"),
        }
    }

    pub fn try_as_pair(&self) -> Option<Ref<Pair>> {
        match self {
            Expr::Pair(pair_handle) => Some(pair_handle.borrow()),
            _ => None,
        }
    }

    pub fn as_sequence(&self) -> Option<&[Expr]> {
        match self {
            Expr::Sequence(expressions) => Some(expressions.as_slice()),
            Expr::List(expressions) => Some(expressions.as_slice()),
            _ => None,
        }
    }

    pub fn as_closure(&self) -> Option<&Handle<Closure>> {
        match self {
            Expr::Closure(handle) => Some(handle),
            _ => None,
        }
    }

    #[inline]
    pub fn repr(&self) -> ExprRepr {
        ExprRepr::new(self)
    }
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Nil
    }
}

impl PartialEq<Expr> for Expr {
    fn eq(&self, other: &Expr) -> bool {
        use Expr::*;

        match (self, other) {
            (Nil, Nil) => true,
            (Void, Void) => true,
            (Bool(a), Bool(b)) => a == b,
            (Number(a), Number(b)) => a == b,
            (String(a), String(b)) => a == b,
            (Ident(a), Ident(b)) => a == b,
            (Keyword(a), Keyword(b)) => a == b,
            (Procedure(a), Procedure(b)) => Rc::ptr_eq(a, b),
            (Closure(a), Closure(b)) => a.ptr_eq(b),
            _ => false,
        }
    }
}

impl From<f64> for Expr {
    #[inline(always)]
    fn from(value: f64) -> Self {
        Expr::Number(value)
    }
}

pub struct ExprRepr<'a> {
    expr: &'a Expr,
}

impl<'a> ExprRepr<'a> {
    fn fmt_expressions(&self, f: &mut fmt::Formatter, expressions: &[Expr]) -> fmt::Result {
        write!(f, "(")?;
        for (idx, expr) in expressions.iter().enumerate() {
            if idx != 0 {
                write!(f, " ")?;
            }
            let repr = ExprRepr { expr };
            write!(f, "{repr}")?;
        }
        write!(f, ")")?;
        Ok(())
    }
}

impl<'a> fmt::Display for ExprRepr<'a> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self.expr {
            Expr::Nil => write!(f, "'()"),
            Expr::Void => write!(f, "#!void"),
            Expr::Bool(boolean) => {
                if *boolean {
                    write!(f, "#t")
                } else {
                    write!(f, "#f")
                }
            }
            Expr::Number(number) => write!(f, "{number}"),
            Expr::String(string) => write!(f, "{string}"),
            Expr::Ident(name) => write!(f, "{name}"),
            Expr::Keyword(keyword) => match keyword {
                Keyword::Dot => write!(f, "."),
            },
            Expr::List(list) => {
                self.fmt_expressions(f, list)?;
                Ok(())
            }
            Expr::Sequence(expressions) => {
                self.fmt_expressions(f, expressions)?;
                Ok(())
            }
            Expr::Procedure(procedure) => {
                write!(f, "<procedure {:?}>", Rc::as_ptr(procedure))
            }
            Expr::Closure(closure) => {
                write!(
                    f,
                    "<procedure {:?}>",
                    Rc::as_ptr(&closure.borrow().procedure_rc())
                )
            }
            Expr::NativeFunc(func) => {
                //  TODO!("keep Rust function name")
                write!(f, "<native-function>")
            }
            unsupported_type => {
                todo!("expression type repr not implemented yet: {unsupported_type:?}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Keyword {
    Dot,
}

pub type NativeFunc = fn(env: &mut Env, args: &[Expr]) -> Result<Expr>;

/// Procedure prototype object.
///
/// This should be treated as immutable, stored as a constant in the environment.
/// It's not identified by a name.
#[derive(Debug)]
pub struct Proc {
    pub(crate) code: Box<[Op]>,

    /// The number of arguments this function accepts.
    pub(crate) sig: Signature,

    pub(crate) constants: Box<[Expr]>,

    /// The number of local variables per call frame that this procedure needs.
    pub(crate) local_count: usize,

    /// The number of up-values that a closure of this procedure will close
    /// over when instantiated.
    pub(crate) up_value_count: usize,

    /// The environment where the procedure was defined.
    ///
    /// Because the procedure is referenced by a closure, and both can
    /// be stored in variables within the environment, a circular reference
    /// is created that would prevent the environment from being dropped.
    pub(crate) env: RcWeak<RefCell<Env>>,
}

/// Procedure signature.
///
/// Describes how many arguments a procedure takes when called.
#[derive(Debug, Clone)]
pub struct Signature {
    /// The fixed number of arguments this procedure accepts.
    pub arity: u8,
    /// Indicates that the procedure can that a variable number of arguments
    /// after its fixed arguments.
    pub variadic: bool,
}

impl Signature {
    pub(crate) const fn new(arity: u8, variadic: bool) -> Self {
        Self { arity, variadic }
    }

    pub(crate) const fn empty() -> Self {
        Self::new(0, false)
    }
}

impl Proc {
    /// Bytecode instructions for this procedure.
    #[inline]
    pub fn bytecode(&self) -> &[Op] {
        &*self.code
    }
}

/// A callable instance of a function.
#[derive(Debug)]
pub struct Closure {
    /// Shared handle to the function definition.
    ///
    /// Procedures are considered immutable after they're compiled,
    /// so we use [`Rc`] directly without the interior mutability
    /// offered by [`Handle`].
    pub(crate) proc: Rc<Proc>,

    // TODO: Change to Box<[UpValue]>
    pub(crate) up_values: Vec<Handle<UpValue>>,
}

impl Closure {
    pub fn new(proc: Rc<Proc>) -> Self {
        Self {
            proc,
            up_values: Vec::new(),
        }
    }

    pub fn with_up_values(proc: Rc<Proc>, up_values: Vec<Handle<UpValue>>) -> Self {
        Self { proc, up_values }
    }

    /// The procedure definition that this closure instances.
    #[inline]
    pub fn procedure(&self) -> &Proc {
        &*self.proc
    }

    /// The procedure definition that this closure instances.
    #[inline]
    pub fn procedure_rc(&self) -> Rc<Proc> {
        self.proc.clone()
    }
}

/// An Up-value is a variable that is referenced within a scope, but is not
/// local to that scope.
#[derive(Debug, Clone)]
pub enum UpValue {
    /// A local variable is an **open** up-value when it is still within scope
    /// and on the operand stack.
    ///
    /// In this case the up-value holds an absolute *stack offset* pointing to the
    /// local variable.
    Open(usize),

    /// A local variable is a **closed** up-value when the closure escapes its
    /// parent scope. The lifetime of those locals extend beyond their scope,
    /// so must be replaced with heap allocated values.
    ///
    /// In this case the up-value holds a *handle* to a heap value.
    Closed(Expr),
}

impl UpValue {
    #[inline]
    pub(crate) fn close(&mut self, value: Expr) {
        // TODO: Must we stop closing a closed up-value?
        *self = UpValue::Closed(value);
    }
}

/// Extensions for the *cons* type. See [`Pair`] and [`Expr::Pair`].
pub trait PairExt {
    fn split_first(&self) -> Option<(Expr, Expr)>;
}
