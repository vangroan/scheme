use crate::env::ConstantId;
use crate::symbol::SymbolId;

#[derive(Debug, Clone)]
pub enum Op {
    /// Push a new `nil` value onto the operand stack.
    PushNil,

    PushTrue,
    PushFalse,
    PushConstant(ConstantId),

    /// Load the variable in the current environment onto the operand stack.
    LoadEnvVar(SymbolId),

    /// Call the Scheme procedure stored in the current environment.
    CallEnvProc {
        symbol: SymbolId,
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
}
