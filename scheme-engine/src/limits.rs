/// Maximum number of local variables, including up-values, per call frame.
///
/// This limitation is from using `u8` as the local ID in bytecode.
/// See [`scheme_engine::opcodes`]
pub const MAX_LOCALS: usize = 1 << 8;

/// Maximum bytecode address that can be stored in a jump instruction.
///
/// Limited by the amount of space in a 32-bit instruction after the opcode.
pub const MAX_JUMP_ADDR: usize = 1 << MAX_JUMP_ADDR_BITS;
pub const MAX_JUMP_ADDR_BITS: usize = 24;
