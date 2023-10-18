use std::mem;
use std::rc::Rc;

use smol_str::SmolStr;

use crate::env::{ConstantId, Env, LocalId};
use crate::error::{Error, Result};
use crate::expr::{Closure, Expr, Keyword, Proc};
use crate::handle::Handle;
use crate::limits::*;
use crate::opcode::Op;
use crate::symbol::SymbolId;

/// Compiles the given top-level expression into bytecode.
///
/// The given environment will be used as the environment
/// of the created procedure.
pub fn compile(env: Handle<Env>, expr: &Expr) -> Result<Handle<Closure>> {
    // Create a new procedure to act as the top level execution context.
    let proc = ProcState::new();

    let mut compiler = Compiler {
        env,
        proc,
        proc_stack: Vec::new(),
        // Compilation starts at the top level of a program.
        context: Context::TopLevel,
        depth: 0,
        locals: Vec::new(),
        stack_offset: 0,
    };

    compiler.compile_expr(expr)?;
    compiler.compile_end()?;

    let (_env, proc) = compiler.take_procedure()?;

    // debug dump the generated bytecode
    println!("bytecode:");
    for (index, op) in proc.code.iter().enumerate() {
        println!("  {index:>6} : {op:?}");
    }

    let closure = Closure::new(Rc::new(proc));

    Ok(Handle::new(closure))
}

struct Compiler {
    /// The current procedure being compiled.
    /// TODO: Once the environment stack is figured out, we could change this to a borrow.
    env: Handle<Env>,

    /// The current procedure being compiled.
    proc: ProcState,

    /// Stack of procedure scopes being compiled.
    proc_stack: Vec<ProcState>,

    /// Keeps track of the scope context as the compiler drills down into expressions.
    ///
    /// This is for the unusual semantics required by special forms.
    ///
    /// See [`Compiler::compile_special_form`] for special forms.
    context: Context,

    /// The current scope depth.
    ///
    /// Used to keep track of which lexical scope local variables are declared in.
    depth: usize,

    /// Stack to keep track of the lexical scoping of local (internal) variables.
    locals: Vec<Local>,

    /// The position (index) in the [`locals`] stack where the current scope's
    /// local variables start.
    ///
    /// Any [`LocalId`] declared within the current scope will be relative to
    /// this position.
    stack_offset: usize,
}

impl Compiler {
    /// Consume the compiler and take the last procedure as the top-level program.
    fn take_procedure(self) -> Result<(Handle<Env>, Proc)> {
        let Self { env, proc, .. } = self;

        // Convert the procedure state to an immutable procedure definition
        // suitable for the virtual machine.
        let proc = Proc {
            code: proc.code.into_boxed_slice(),
            // The top level procedures never take arguments.
            arity: 0,
            variadic: false,
            constants: proc.constants.into_boxed_slice(),
            // By storing the procedure in the environment
            // we've created a circular reference.
            env: env.downgrade(),
        };

        Ok((env, proc))
    }

    /// Set the current scope context for the duration of the given closure.
    fn context<T, F>(&mut self, ctx: Context, block: F) -> Result<T>
    where
        F: FnOnce(&mut Compiler) -> Result<T>,
    {
        let old_context = self.context;
        self.context = ctx;
        let result = block(self);
        self.context = old_context;

        result
    }

    /// Create a new lexical scope, storing the current scoping
    /// depth and replacing it with the given scope.
    ///
    /// Local variables will be declared in the new scope, shadowing
    /// outer ones. Accessing outer variables will create up-values.
    ///
    /// Once the closure returns, the original scope is restored.
    ///
    /// # Returns
    ///
    /// Returns the value returned from the closure, as well as
    /// the newly compiled procedure.
    fn scope<T, F>(&mut self, block: F) -> Result<T>
    where
        F: FnOnce(&mut Compiler) -> Result<T>,
    {
        self.depth += 1;
        let result = block(self);
        self.depth -= 1;

        result
    }

    /// Create a new procedure and a lexical scope to encapsulate it.
    fn proc_scope<T, F>(&mut self, block: F) -> Result<(T, ProcState)>
    where
        F: FnOnce(&mut Compiler) -> Result<T>,
    {
        let prev_proc = mem::replace(&mut self.proc, ProcState::new());
        let result = self.scope(|compiler| Ok(block(compiler)))?;
        let new_proc = mem::replace(&mut self.proc, prev_proc);

        result.map(|r| (r, new_proc))
    }

    /// Compile a sequence of expressions.
    ///
    /// The given expression in the `expr` argument must be a list.
    ///
    /// Only the result of the final expression is left
    /// on the operand stack during evaluation.
    fn compile_sequence(&mut self, expr: &Expr) -> Result<()> {
        if !matches!(expr, Expr::Sequence(_)) {
            return Err(Error::Reason("expected a sequence or nil".to_string()));
        }

        let expressions = expr.as_slice().unwrap();

        if let Some((last, preceding)) = expressions.split_last() {
            for expr in preceding {
                self.compile_expr(expr)?;

                // Discard the result values of the preceding expressions.
                self.proc.emit_op(Op::Pop);
            }

            self.compile_expr(last)?;
        }

        Ok(())
    }

    /// Compile a single expression.
    ///
    /// Returns the number of resulting values the expression's
    /// evaluation would leave on the operand stack during runtime.
    fn compile_expr(&mut self, expr: &Expr) -> Result<()> {
        println!("compiler::compile_expr({expr:?})");

        match expr {
            // Nil literal
            Expr::Nil => {
                self.proc.emit_op(Op::PushNil);
            }
            // Number literal
            Expr::Number(number) => {
                let constant_id = self.add_constant(expr.clone());
                self.proc.emit_op(Op::PushConstant(constant_id));
            }
            // Boolean literal
            Expr::Bool(boolean) => {
                let op = if *boolean {
                    Op::PushTrue
                } else {
                    Op::PushFalse
                };
                self.proc.emit_op(op);
            }
            Expr::Ident(ident) => {
                // Attempt to access a variable by identifier.
                self.compile_access(ident.as_str())?;
            }
            Expr::List(list) => {
                self.compile_form(list.as_slice())?;
            }
            Expr::Sequence(_) => {
                self.compile_sequence(expr)?;
            }
            _ => todo!("compile_expr: {expr:?}"),
        }

        Ok(())
    }

    fn compile_end(&mut self) -> Result<()> {
        self.proc.emit_op(Op::End);
        Ok(())
    }

    /// Compile variable access.
    ///
    /// # Return
    ///
    /// Returns the symbol for the location where the variable is stored.
    fn compile_access(&mut self, name: &str) -> Result<Variable> {
        // First attempt to resolve the variable in a local scope,
        // an outer scope, then the enclosing environment.
        for local in self.locals.iter().rev() {
            if name == local.name {
                if self.depth != local.depth {
                    todo!("up-values for locals outside of the current scope");
                }

                self.proc.emit_op(Op::LoadLocalVar(local.id));

                return Ok(Variable::Local(local.id));
            }
        }

        // If the variable cannot be found in the locals of the lexical scopes,
        // then we fall back onto the enclosing environment.
        let symbol = self
            .env
            .borrow()
            .resolve_var(name)
            .ok_or_else(|| Error::Reason(format!("unbound variable {name:?}")))?;

        // VM: Load the variable from the environment onto the operand stack.
        self.proc.emit_op(Op::LoadEnvVar(symbol));

        Ok(Variable::Symbol(symbol))
    }

    fn compile_form(&mut self, list: &[Expr]) -> Result<()> {
        if self.compile_special_form(list)? {
            Ok(())
        } else {
            // The default s-expression form is a procedure call.
            self.compile_call(list)
        }
    }

    /// Attempt to compile a special form.
    ///
    /// Special forms are expression that follow unusual evaluation rules.
    ///
    /// They are implemented as compiler intrinsics that generate inlined bytecode.
    ///
    /// # Return
    ///
    /// Returns `true` if the form is considered a special form.
    /// Returns `false` if the form cannot be identified, with no bytecode emitted.
    fn compile_special_form(&mut self, list: &[Expr]) -> Result<bool> {
        // TODO: When the identifiers themselves are evaluated like variables then a special error must be raised.
        // > define
        // error: fundamental name cannot be used as a variable.

        if let Some((Expr::Ident(operator), rest)) = list.split_first() {
            match operator.as_str() {
                "define" => {
                    self.compile_define_form(rest)?;
                    Ok(true)
                }
                "lambda" => {
                    self.compile_lambda_form(rest)?;
                    Ok(true)
                }
                "let" => {
                    todo!("let form")
                }
                "let*" => {
                    todo!("let* form")
                }
                "letrec" => {
                    todo!("letrec form")
                }
                "fluid-let" => {
                    todo!("fluid-let form")
                }
                "set!" => {
                    todo!("set! form")
                }
                "quote" => {
                    todo!("quote form")
                }
                _ => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn compile_call(&mut self, list: &[Expr]) -> Result<()> {
        println!("compiler::compile_call({list:?})");

        if list.is_empty() {
            return Err(Error::Reason("ill-formed expression".to_string()));
        }

        match &list[0] {
            Expr::Ident(ident) => {
                let rest = &list[1..];
                println!("rest of arguments: {rest:?}");

                // TODO: Walk scope in reverse to find variable.

                // Lookup procedure using the first atom of the sequence.
                let symbol = self
                    .env
                    .borrow()
                    .resolve_var(ident.as_str())
                    .ok_or_else(|| Error::Reason(format!("unbound variable {ident:?}")))?;
                let value = self.env.borrow().get_var(symbol).cloned().unwrap();
                self.proc.emit_op(Op::LoadEnvVar(symbol));

                // TODO: Variadic procedures take the rest of their arguments as list. We need to store signature information to accomplish this.
                for arg in rest {
                    self.compile_expr(arg)?;
                }

                match value {
                    Expr::Closure(_) => todo!("call closure"),
                    Expr::NativeFunc(_) => self.proc.emit_op(Op::CallNative {
                        arity: rest.len() as u8,
                    }),
                    Expr::Procedure(_) => {
                        // Procedures are not stored in variables, but are rather
                        // wrapped in closures when defined.
                        //
                        // This adds overhead when creating the procedure, but simplifies
                        // the interpreter implementation because the VM only needs to
                        // know how to call closures.
                        panic!("procedures cannot be called directly")
                    }
                    _ => todo!("call type not supported yet"),
                }

                Ok(())
            }
            _ => Err(Error::Reason("operator is not a procedure".to_string())),
        }
    }

    /// Compile the `define` special form.
    ///
    /// A fundamental special form that defines a variable in the current environment.
    ///
    /// Definitions can appear in two places, at the top level of a program
    /// where they are called **top-level definitions** or at the beginning
    /// of a body where they are called **internal definitions**. It is an
    /// error to place a define anywhere else.
    ///
    /// Definitions also have a second form that also defines a procedure.
    ///
    /// # Return
    ///
    /// Returns the [`SymbolId`] of the defined variable.
    fn compile_define_form(&mut self, rest: &[Expr]) -> Result<SymbolId> {
        // TODO: `define` expression may only appear in specific places.
        match rest
            .get(0)
            .ok_or_else(|| Error::Reason("identifier expected".to_string()))?
        {
            Expr::Ident(var_name) => {
                // Variables can be redefined
                let symbol = self.env.borrow_mut().intern_var(var_name);

                // Define body is an expression, but may be omitted.
                // TODO: Do we need an undefined, unspecified or void value?
                let body = rest.get(1).unwrap_or(&Expr::Nil);

                // This expression leaves a value on the stack.
                self.compile_expr(body)?;

                match self.context {
                    Context::TopLevel => {
                        self.proc.emit_op(Op::StoreEnvVar(symbol));
                        self.proc.emit_op(Op::Pop);

                        // Define evaluates to a #!void value.
                        //
                        // It is the responsibility of the few contexts where define
                        // is allowed to clean this void off the stack.
                        self.proc.emit_op(Op::PushVoid);
                    }
                    Context::BodyStart => {
                        let local_id = self.declare_local(var_name.as_str())?;
                        self.proc.emit_op(Op::StoreLocalVar(local_id));
                    }
                    Context::BodyRest => {
                        return Err(Error::Reason("ill-formed special form: define must appear at top-level or first in body".to_string()));
                    }
                }
                // FIXME: StoreEnvVar does not pop, and we're leaving the pop to `compile_sequence`, but define is not suppose to return a value.

                Ok(symbol)
            }
            Expr::List(_formals) => {
                todo!("procedure definition")
            }
            _ => Err(Error::Reason("ill-formed special form".to_string())),
        }
    }

    /// Compile the `lambda` special form.
    ///
    /// ```scheme
    /// (lambda <formal> <body>)
    /// (lambda (<formals>) <body>)
    /// (lambda (<formals> . <rest>) <body>)
    /// ```
    fn compile_lambda_form(&mut self, rest: &[Expr]) -> Result<()> {
        if let Some((formals, rest)) = rest.split_first() {
            let (_, proc_state) = self.proc_scope(|compiler| {
                match formals {
                    // If the formals are a single identifier, then that
                    // identifier is the "rest" variadic parameter.
                    //
                    // The lambda is completely variadic and all arguments will be
                    // passed as a list bound to this formal.
                    Expr::Ident(_name) => {
                        todo!()
                    }
                    // The formal parameter list is a list of identifiers
                    // to which the call arguments will be bound.
                    //
                    // The lambda's arity is determined by the number of
                    // identifiers in this list.
                    //
                    // The parameters may be followed by a dot(.) keyword, after
                    // which the "rest" parameters will be bound as a list.
                    Expr::List(list) => {
                        compiler.proc.arity = 0;
                        let mut variadic: bool = false;

                        for param in list {
                            match param {
                                Expr::Ident(name) => {
                                    // Declare bindings in this scope so the
                                    // arguments can be referenced by name
                                    // in the lambda body.
                                    compiler.declare_local(name.as_str())?;

                                    if variadic {
                                        break;
                                    } else {
                                        compiler.proc.arity += 1;
                                    }
                                }
                                Expr::Keyword(Keyword::Dot) => {
                                    // When we encounter a dot(.) the succeeding parameter
                                    // is the binding to the variadic list.
                                    variadic = true;
                                }
                                _ => {
                                    return Err(Error::Reason(
                                        "parameter must be an identifier".to_string(),
                                    ))
                                }
                            }
                        }

                        compiler.compile_body(rest)?;

                        Ok(())
                    }
                    _ => Err(Error::Reason("parameter must be an identifier".to_string())),
                }
            })?;

            println!("procedure compiled:");
            for (index, op) in proc_state.code.iter().enumerate() {
                println!("  {index:>6} : {op:?}");
            }

            let proc = proc_state.into_proc(self.env.clone());

            /// The procedure definition is stored as a constant in the outer environment.
            let constant_id = self.add_constant(Expr::Procedure(Rc::new(proc)));
            self.proc.emit_op(Op::CreateClosure(constant_id));

            Ok(())
        } else {
            Err(Error::Reason(
                "ill-formed special form: lambda expects formal parameters followed by a body"
                    .to_string(),
            ))
        }
    }

    /// Compile the body of a `define`, `lambda`, `let`, etc...
    fn compile_body(&mut self, rest: &[Expr]) -> Result<()> {
        self.context(Context::BodyStart, |compiler| {
            // Compile the start of a body.
            //
            // This is where definitions are allowed. We keep compiling until a
            // non-definition expression is encountered.
            let mut body_expressions = rest;

            for expr in rest {
                if let Expr::List(list) = expr {
                    if let Some((Expr::Ident(name), def_rest)) = list.split_first() {
                        match name.as_str() {
                            "define" => {
                                compiler.compile_define_form(def_rest)?;
                                body_expressions = &rest[1..];
                            }
                            "define-syntax" => {
                                todo!("define-syntax")
                            }
                            _ => break,
                        }
                    }
                } else {
                    break;
                }
            }

            compiler.context(Context::BodyRest, |compiler| {
                // No further definitions not allowed here.

                if let Some((last, preceding)) = body_expressions.split_last() {
                    for expr in preceding {
                        compiler.compile_expr(expr)?;

                        // Discard the result values of the preceding expressions.
                        compiler.proc.emit_op(Op::Pop);
                    }

                    // Only the last expression's result is left on the stack.
                    compiler.compile_expr(last)?;
                }

                Ok(())
            })?;

            Ok(())
        })
    }

    /// Add a constant value to the current environment.
    ///
    /// Returns the [`ConstantId`] identifying its location.
    ///
    /// Does not emit a load operation.
    fn add_constant(&mut self, value: Expr) -> ConstantId {
        match self.proc.constants.iter().position(|el| el == &value) {
            Some(index) => ConstantId::new(index as u16),
            None => {
                let next_index = self.proc.constants.len();
                self.proc.constants.push(value);
                ConstantId::new(next_index as u16)
            }
        }
    }

    /// Add a local variable to the the current scope.
    ///
    /// Local variables defined in a scope will shadow variables
    /// with the same name defined in outer scopes.
    ///
    /// # Errors
    ///
    /// Returns an error if the current scope is the top-level.
    /// Locals are only for bodies.
    ///
    /// Returns an error if a local with the given name already exists
    /// in the current scope.
    ///
    /// # Returns
    ///
    /// A new [`LocalId`] identifying the local variable's location,
    /// relative to the start of the call frame during runtime.
    fn declare_local(&mut self, name: &str) -> Result<LocalId> {
        // Resolve local by scanning the lexical stack backwards.
        for local in self.locals.iter().rev() {
            if local.depth != self.depth {
                // We've left our scope and stop the scan.
                // Any locals with the same name will now be shadowed.
                break;
            }

            if name == local.name {
                return Err(Error::Reason(format!(
                    "duplicate definition of local variable \"{name}\""
                )));
            }
        }

        let index = self.locals.len() - self.stack_offset;

        if index >= MAX_LOCALS {
            return Err(Error::Reason(format!(
                "number of local variables in scope exceeds maximum of {MAX_LOCALS}"
            )));
        }

        let local_id = LocalId::new(index as u8);
        self.locals.push(Local {
            id: local_id,
            name: SmolStr::from(name),
            depth: self.depth,
        });
        Ok(local_id)
    }
}

/// Mutable bookkeeping for compiling a procedure.
struct ProcState {
    /// Generated result bytecode.
    code: Vec<Op>,
    arity: usize,
    variadic: bool,
    constants: Vec<Expr>,
}

impl ProcState {
    fn new() -> Self {
        Self {
            code: Vec::new(),
            arity: 0,
            variadic: false,
            constants: Vec::new(),
        }
    }

    fn emit_op(&mut self, op: Op) {
        self.code.push(op)
    }

    fn into_proc(self, env: Handle<Env>) -> Proc {
        let Self {
            code,
            arity,
            variadic,
            constants,
        } = self;

        Proc {
            code: code.into_boxed_slice(),
            arity,
            variadic,
            constants: constants.into_boxed_slice(),
            env: env.downgrade(),
        }
    }
}

#[derive(Debug)]
enum Variable {
    Local(LocalId),
    /// The symbol identifying the location where the variable is stored int he current environment.
    Symbol(SymbolId),
}

/// Slot for a local variable on the lexical stack.
#[derive(Debug, Clone)]
struct Local {
    id: LocalId,
    name: SmolStr,
    /// The depth of the scope where the variable was declared.
    depth: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    TopLevel,
    BodyStart,
    BodyRest,
}

impl Context {
    fn is_top_level(&self) -> bool {
        matches!(self, Self::TopLevel)
    }

    fn is_body_start(&self) -> bool {
        matches!(self, Self::BodyStart)
    }

    fn is_body_rest(&self) -> bool {
        matches!(self, Self::BodyRest)
    }
}
