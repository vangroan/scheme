//! Execution environment.

pub struct Env {
    /// Table of values which do not change during runtime.
    ///
    /// Includes literals like booleans, numbers and strings, but
    /// also defined functions.
    constants: (),

    /// Table of values which can be mutated during runtime.
    ///
    /// Does not include functions, but can include closure instances.
    variables: (),
}
