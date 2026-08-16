use thiserror::Error;

#[derive(Debug, Error)]
pub enum LaxError {
    #[error("{0}")]
    Msg(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl LaxError {
    pub fn msg(m: impl Into<String>) -> Self {
        Self::Msg(m.into())
    }
}

impl From<LaxError> for String {
    fn from(value: LaxError) -> Self {
        value.to_string()
    }
}

pub type LaxResult<T> = Result<T, LaxError>;
