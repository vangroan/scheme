/// Maximum number of local variables, including up-values, per call frame.
///
/// This limitation is from using `u8` as the local ID in bytecode.
/// See [`scheme_engine::opcodes`]
pub const MAX_LOCALS: usize = 1 << 8;
