//! Virtual machine.

use crate::error::{Error, Result};
use crate::expr::{Closure, Expr, Proc};
use crate::handle::Handle;
use crate::opcode::Op;
use std::rc::Rc;

pub fn eval(closure: Handle<Closure>) -> Result<Expr> {
    let mut vm = Vm::new();
    vm.run(closure)
}

struct Vm {
    /// The operand stack.
    operand: Vec<Expr>,

    /// The call stack.
    frames: Vec<CallFrame>,
}

struct CallFrame {
    /// The closure being executed.
    closure: Handle<Closure>,

    /// The index into the machine's operand stack where this frame's working set starts.
    stack_offset: usize,
}

/// A procedure action is a message from the instruction
/// loop to the outer control to change the context.
enum ProcAction {
    /// Call a new closure and push it onto the top of the callstack.
    Call,
    /// Call a new closure and replace the top of the callstack with it.
    TailCall,
    /// Return an expression result.
    Return(Expr),
}

impl Vm {
    fn new() -> Self {
        Self {
            operand: Vec::new(),
            frames: Vec::new(),
        }
    }

    fn run(&mut self, closure: Handle<Closure>) -> Result<Expr> {
        if !self.frames.is_empty() {
            // The machine is already executing something, so
            // a new closure cannot be called.
            //
            // This, or something like it, will be needed once we implement continuations.
            return Err(Error::Reason(
                "machine is already executing a closure".to_string(),
            ));
        }

        self.frames.push(CallFrame {
            closure,
            stack_offset: 0,
        });

        run_interpreter(self)
    }
}

/// Run the interpreter loop.
fn run_interpreter(vm: &mut Vm) -> Result<Expr> {
    // Pull the top call frame off the stack, to allow
    // the loop to work with both the owning VM and call frame
    // with minimum borrow puzzles.
    let mut frame = vm
        .frames
        .pop()
        .expect("vm must have at least one call frame");

    match run_instructions(vm, &mut frame)? {
        ProcAction::Call => todo!("procedure call"),
        ProcAction::TailCall => todo!("tail call"),
        ProcAction::Return(value) => {
            // NOTE: Keep the frame off the stack for an implicit pop.
            return Ok(value);
        }
    }
}

/// Run the bytecode instruction loop.
fn run_instructions(vm: &mut Vm, frame: &mut CallFrame) -> Result<ProcAction> {
    // Pull relevant state into flat local variables to reduce the
    // overhead of jumping pointers and bookkeeping of borrowing objects.
    let mut closure_rc = frame.closure.clone();
    let closure_ref = closure_rc.borrow_mut();
    let proc_rc = closure_ref.procedure().clone();
    let proc = &*proc_rc;
    let env_rc = proc.env.upgrade().unwrap();
    let env = &mut *env_rc.borrow_mut();
    let ops = proc.bytecode();
    let mut pc: usize = 0;

    loop {
        let op = ops[pc].clone();
        pc += 1;

        match op {
            Op::PushNil => {
                vm.operand.push(Expr::Nil);
            }
            Op::PushTrue => {
                vm.operand.push(Expr::Bool(true));
            }
            Op::PushFalse => {
                vm.operand.push(Expr::Bool(false));
            }

            Op::Return => todo!("return"),
            Op::LoadEnvVar(symbol) => {
                let value = env.get_var(symbol).cloned().unwrap_or(Expr::Nil);
                vm.operand.push(value);
            }
            Op::StoreEnvVar(symbol) => {
                let value = vm.operand.last().cloned().unwrap_or(Expr::Nil);
                env.set_var(symbol, value)?;
            }
            Op::PushConstant(constant_id) => {
                let value = proc
                    .constants
                    .get(constant_id.as_usize())
                    .cloned()
                    .unwrap_or(Expr::Nil);
                vm.operand.push(value);
            }
            Op::Pop => {
                let _ = vm.operand.pop();
            }
            Op::CallEnvProc { arity } => todo!("call procedure"),

            // Call a native function.
            //
            // The stack must be prepared with a variable holding the function pointer,
            // followed by all the arguments to be passed to the call.
            Op::CallNative { arity } => {
                // TODO: Support variadic procedures
                let lo = vm.operand.len() - arity as usize;

                // The value just below the arguments is expected to hold the callable.
                let callable = &vm.operand[lo - 1];
                let args = &vm.operand[lo..];

                let value = match callable {
                    Expr::NativeFunc(func) => func(env, args)?,
                    Expr::Procedure(_) => todo!("other call types not implemented yet"),
                    _ => return Err(Error::Reason("invalid callable type".to_string())),
                };

                vm.operand.truncate(lo - 1);
                vm.operand.push(value);
            }
            Op::End => {
                let value = vm.operand.pop().unwrap_or(Expr::Nil);
                return Ok(ProcAction::Return(value));
            }
        }
    }
}

/// Call a procedure or native function.
#[inline]
fn call(vm: &mut Vm) -> Result<ProcAction> {
    todo!()
}
