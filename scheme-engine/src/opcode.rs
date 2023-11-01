use crate::env::{ConstantId, LocalId, ProcId, UpValueId};
use crate::limits::*;
use crate::symbol::SymbolId;

#[derive(Debug, Clone)]
pub enum Op {
    Bail,
    /// Push a new `nil` value onto the operand stack.
    PushNil,
    /// Push a new `#!void` value onto the operand stack.
    PushVoid,

    PushTrue,
    PushFalse,
    PushConstant(ConstantId),

    /// Remove and discard the top value off the stack.
    Pop,

    /// Jump to the specified absolute address if the top stack value is #f
    JumpFalse(JumpAddr),

    /// Unconditional jump to the specified absolute address.
    Jump(JumpAddr),

    /// Return from a procedure call.
    Return,

    /// Load the variable in the current environment onto the operand stack.
    LoadEnvVar(SymbolId),

    /// Store the value on the top of operand stack into the current environment by
    /// copying it into the variable with the given symbol.
    ///
    /// Does not implicitly pop the value off the stack.
    StoreEnvVar(SymbolId),

    LoadUpValue(UpValueId),
    StoreUpValue(UpValueId),

    LoadLocalVar(LocalId),

    /// Store the value on the top of the operand stack into the local
    /// variable at the given location.
    ///
    /// Does not implicitly pop the value off the stack.
    StoreLocalVar(LocalId),

    /// Capture a variable as an up-value for the coming closure creation. See [`Op::CreateClosure`]
    CaptureValue(UpValueOrigin),

    /// Instantiate a new closure object.
    ///
    /// The constant ID argument is the location of the procedure definition
    /// that this closure instantiates.
    ///
    /// This instruction is preceded by zero or more  [`Op::CaptureValue`] operations
    /// that setup the stack with up-values.
    CreateClosure(ProcId),

    /// Call a closure instance instance.
    CallClosure {
        arity: u8,
    },

    /// Call a native Rust function pointer, stored in the current environment.
    ///
    /// The operand stack should first have a value of type [`Expr::NativeFunc`],
    /// then on top of that the arguments with the first argument at the bottom,
    /// and the last argument at the top.
    CallNative {
        arity: u8,
    },

    /// End of bytecode sentinel.
    End,
}

/// Absolute bytecode address for jumps.
#[derive(Debug, Clone)]
pub struct JumpAddr(pub(crate) [u8; 3]);

impl JumpAddr {
    pub(crate) fn new(index: usize) -> JumpAddr {
        if index >= MAX_JUMP_ADDR {
            panic!("maximum jump address size of {MAX_JUMP_ADDR_BITS} bits exceeded")
        }

        let [a, b, c, ..] = index.to_le_bytes();
        JumpAddr([a, b, c])
    }

    /// Create a jump to address 0.
    pub(crate) const fn zero() -> JumpAddr {
        JumpAddr([0; 3])
    }

    pub fn as_usize(&self) -> usize {
        let [a, b, c] = self.0;
        usize::from_le_bytes([a, b, c, 0, 0, 0, 0, 0])
    }
}

/// Indicates how far from the local scope the up-value originated.
///
/// An open up-value pointing to the immediate parent scope has its
/// value in that parent's local variables.
///
/// An open up-value with a value from beyond that, has to point to
/// the parent scope's up-value list.
///
/// During runtime, outer scopes are not guaranteed to be on the
/// call stack when a closure is instantiated, because multiple
/// closures can be nested and returned.
///
/// In this example z is local, y is an up-value pointing to a parent's local (origin `Parent`),
/// and x is an up-value pointing to a parent's up-value (origin `Outer`) which in turn
/// points to the grand-parent's local.
///
/// ```scheme
/// (lambda (x)      ;; outer
///   (lambda (y)    ;; parent
///     (lambda (z)  ;; local
///       (+ x y z)
///   )))
/// ```
///
/// Up-values from outer scopes are copied down into inner scopes,
/// their handles shared so "closing" will reflect in all, effectively
/// *flattening* the closures.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpValueOrigin {
    /// UpValue is located in parent's local variables.
    Parent(LocalId),
    /// UpValue is located in parent's up-value list.
    Outer(UpValueId),
}

impl UpValueOrigin {
    #[inline]
    fn is_parent(&self) -> bool {
        matches!(self, UpValueOrigin::Parent(_))
    }

    #[inline]
    fn is_outer(&self) -> bool {
        matches!(self, UpValueOrigin::Outer(_))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_opcode_size() {
        println!(
            "size_of::<Op>() -> {} Bytes, {} bits",
            std::mem::size_of::<Op>(),
            std::mem::size_of::<Op>() * 8
        );
    }

    #[test]
    fn test_jump_addr() {
        let addr = JumpAddr::new(787199);
        assert_eq!(addr.0, [255, 2, 12]);

        let index = addr.as_usize();
        assert_eq!(index, 787199);
    }
}
