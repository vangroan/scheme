use crate::error::Result;

pub fn eval_source(source: &str) -> Result<()> {
    let _ = crate::parser::parse(source);

    Ok(())
}
