//! Virtual machine.

use crate::error::{Error, Result};
use crate::expr::{Closure, Expr};
use crate::handle::Handle;
use crate::opcode::Op;
use std::mem;

pub fn eval(closure: Handle<Closure>) -> Result<Expr> {
    let mut vm = Vm::new();
    let result = vm.run(closure);
    println!("VM eval stack: {:?}", vm.operand);
    result
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

    /// Saved program counter, so the this frame can resume after control is returned.
    pc: usize,
}

/// A procedure action is a message from the instruction
/// loop to the outer control to change the context.
enum ProcAction {
    /// Call a new closure and push it onto the top of the callstack.
    Call(Handle<Closure>, usize),
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

        // For consistency with closure call convention, keep a handle
        // to this closure on the stack.
        self.operand.push(Expr::Closure(closure.clone()));

        self.frames.push(CallFrame {
            closure,
            stack_offset: self.operand.len(),
            pc: 0,
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

    loop {
        match run_instructions(vm, &mut frame)? {
            ProcAction::Call(closure, stack_offset) => {
                let new_frame = CallFrame {
                    closure,
                    stack_offset,
                    pc: 0,
                };

                let old_frame = mem::replace(&mut frame, new_frame);
                vm.frames.push(old_frame);
            }
            ProcAction::TailCall => todo!("tail call"),
            ProcAction::Return(value) => {
                // NOTE: Keep the frame off the stack for an implicit pop.

                // The closure that was called will be on the stack just below the arguments.
                vm.operand.truncate(frame.stack_offset - 1);

                return Ok(value);
            }
        }
    }
}

/// Run the bytecode instruction loop.
fn run_instructions(vm: &mut Vm, frame: &mut CallFrame) -> Result<ProcAction> {
    // Pull relevant state into flat local variables to reduce the
    // overhead of jumping pointers and bookkeeping of borrowing objects.
    let mut closure_rc = frame.closure.clone();
    // let closure_ref = closure_rc.borrow_mut();
    // let proc_rc = closure_ref.procedure().clone();
    let proc_rc = closure_rc.borrow().procedure_rc().clone();
    let proc = &*proc_rc;
    let env_rc = proc.env.upgrade().unwrap();
    let env = &mut *env_rc.borrow_mut();
    let ops = proc.bytecode();
    let mut pc: usize = frame.pc;

    println!("eval stack: {:?}", vm.operand);

    loop {
        let op = ops[pc].clone();
        pc += 1;

        match op {
            Op::PushNil => {
                vm.operand.push(Expr::Nil);
            }
            Op::PushVoid => {
                vm.operand.push(Expr::Void);
            }
            Op::PushTrue => {
                vm.operand.push(Expr::Bool(true));
            }
            Op::PushFalse => {
                vm.operand.push(Expr::Bool(false));
            }

            Op::Return => {
                let value = vm.operand.pop().unwrap_or(Expr::Nil);
                return Ok(ProcAction::Return(value));
            }
            Op::LoadEnvVar(symbol) => {
                let value = env.get_var(symbol).cloned().unwrap_or(Expr::Nil);
                vm.operand.push(value);
            }
            Op::StoreEnvVar(symbol) => {
                let value = vm.operand.last().cloned().unwrap_or(Expr::Nil);
                env.set_var(symbol, value)?;
                // don't pop
            }
            Op::LoadLocalVar(local_id) => {
                let value = vm
                    .operand
                    .get(frame.stack_offset + local_id.as_usize())
                    .cloned()
                    .unwrap_or(Expr::Nil);
                vm.operand.push(value);
            }
            Op::StoreLocalVar(local_id) => {
                let value = vm.operand.last().cloned().unwrap_or(Expr::Nil);
                vm.operand[frame.stack_offset + local_id.as_usize()] = value;
                // don't pop
            }
            Op::PushConstant(constant_id) => {
                println!("push constant {constant_id:?}");
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
            Op::CreateClosure(constant_id) => {
                // The constant stores the procedure definition to instantiate.
                let value = proc
                    .constants
                    .get(constant_id.as_usize())
                    .cloned()
                    .unwrap_or(Expr::Nil);

                match value {
                    Expr::Procedure(proc) => {
                        let closure = Closure::new(proc);
                        let closure_handle = Handle::new(closure);
                        vm.operand.push(Expr::Closure(closure_handle));
                    }
                    _ => {
                        return Err(Error::Reason("expected procedure definition".to_string()));
                    }
                }
            }
            Op::CallClosure { arity } => {
                let lo = vm.operand.len() - arity as usize;

                // The value just below the arguments is expected to hold the callable.
                let callable = &vm.operand[lo - 1];
                let args = &vm.operand[lo..];

                return match callable {
                    Expr::Closure(closure) => Ok(ProcAction::Call(closure.clone(), lo)),
                    _ => Err(Error::Reason("invalid callable type".to_string())),
                };
            }

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

                println!("Op::CallNative, eval stack: {:?}", vm.operand);

                let value = match callable {
                    // Native call does not unwind the Scheme call stack to push a frame.
                    //
                    // It simply calls into Rust from within the instruction loop.
                    Expr::NativeFunc(func) => {
                        let value = func(env, args)?;

                        vm.operand.truncate(lo - 1);
                        vm.operand.push(value);
                    }
                    // A Scheme closure call must unwind the stack to push a new frame,
                    // to avoid a borrow puzzle.
                    Expr::Closure(closure) => {
                        // Save the program counter from this Rust stack frame to
                        // the Scheme frame so we can resume the frame after control
                        // is returned.
                        frame.pc = pc;

                        return Ok(ProcAction::Call(closure.clone(), lo));
                    }
                    Expr::Procedure(_) => todo!("other call types not implemented yet"),
                    _ => return Err(Error::Reason("invalid callable type".to_string())),
                };
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
