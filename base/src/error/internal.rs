use thiserror::Error;
#[derive(Error, Debug)]
pub(crate) enum InternalErr<T> {
    #[error("Ran into {err} when performed operation {op} on {on}")]
    IllegalOperation { op: String, on: T, err: String },
    #[error("Attempt to access idx {idx}. Err: {msg}")]
    IllegalIndexAccess { idx: usize, msg: String },
    #[error("Illegal State: {msg}")]
    IllegalState { msg: String },
}
