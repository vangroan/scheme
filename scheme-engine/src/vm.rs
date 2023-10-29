//! Virtual machine.

use crate::error::{Error, Result};
use crate::expr::{Closure, Expr, UpValue};
use crate::handle::Handle;
use crate::opcode::{Op, UpValueOrigin};
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

    /// These are the *open* up-values belonging to all closures that have captured
    /// local variables belonging to this call frame. They are shared with the closure's
    /// heap space so that closing them reflects within the closure when it escapes.
    ///
    /// Before this frame is popped off the call stack, all its captured locals must
    /// be copied into the up-values, and the up-values closed.
    up_values: Vec<Handle<UpValue>>,

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
            up_values: Vec::new(),
            pc: 0,
        });

        run_interpreter(self)
    }

    fn parent_frame(&self) -> Option<&CallFrame> {
        // During the instruction loop the current call frame is not
        // in the `frames` stack, but owned by the Rust stack.
        //
        // The top frame in the stack is the previous, parent frame.
        self.frames.last()
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
                    up_values: Vec::new(),
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
    println!("eval stack: {:?}", vm.operand);

    // Pull relevant state into flat local variables to reduce the
    // overhead of jumping pointers and bookkeeping of borrowing objects.
    let mut closure_rc = frame.closure.clone();
    // let closure_ref = closure_rc.borrow_mut();
    // let proc_rc = closure_ref.procedure().clone();
    let proc_rc = closure_rc.borrow().procedure_rc().clone();
    let proc = &*proc_rc;
    let closure = &mut *closure_rc.borrow_mut();
    let env_rc = proc.env.upgrade().unwrap();
    let env = &mut *env_rc.borrow_mut();
    let ops = proc.bytecode();
    let mut pc: usize = frame.pc;

    loop {
        let op = ops[pc].clone();
        pc += 1;

        match op {
            Op::Bail => {
                panic!("Bail!")
            }
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
            Op::LoadUpValue(up_value_id) => {
                match closure.up_values[up_value_id.as_usize()].borrow().clone() {
                    UpValue::Open(stack_pos) => {
                        let value = vm.operand[stack_pos].clone();
                        vm.operand.push(value);
                    }
                    UpValue::Closed(value) => {
                        vm.operand.push(value);
                    }
                }
            }
            Op::StoreUpValue(up_value_id) => {
                todo!()
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
            Op::CaptureValue(_) => {
                unreachable!("capture-value must only be processed by closure creation")
            }
            Op::CreateClosure(proc_id) => {
                println!("create closure {proc_id:?}");
                // The constant stores the procedure definition to instantiate.
                let prototype = env
                    .procedures
                    .get(proc_id.as_usize())
                    .cloned()
                    .ok_or_else(|| Error::Reason("expected procedure definition".to_string()))?;

                // Read the capture arguments.
                let mut up_values = Vec::new();

                println!("program counter: {pc}");
                for i in 0..prototype.up_value_count {
                    println!("processing argument {i}");
                    let op = ops[pc].clone();
                    match op {
                        Op::CaptureValue(origin) => {
                            match origin {
                                // Create a new up-value pointing to a local variable
                                // in the current scope.
                                UpValueOrigin::Parent(local_id) => {
                                    up_values.push(Handle::new(UpValue::Open(
                                        frame.stack_offset + local_id.as_usize(),
                                    )));
                                }
                                // Share a handle to an existing up-value.
                                UpValueOrigin::Outer(up_value_id) => {
                                    up_values
                                        .push(closure.up_values[up_value_id.as_usize()].clone());
                                }
                            }
                        }
                        unexpected_op => {
                            return Err(Error::Reason(format!(
                                "invalid capture-value argument instruction: {unexpected_op:?}"
                            )));
                        }
                    }
                    pc += 1;
                }
                println!("program counter: {pc}");

                let closure = Closure::with_up_values(prototype, up_values);
                let closure_handle = Handle::new(closure);
                vm.operand.push(Expr::Closure(closure_handle));
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
