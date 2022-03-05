use std::fmt;

pub type BMResult<T> = std::result::Result<T, BMError>;

#[derive(Debug)]
pub enum BMError {
    LoadingError {
        msg: String
    },
}

impl std::error::Error for BMError {}

impl From<std::io::Error> for BMError {
    fn from(err: std::io::Error) -> Self {
        BMError::LoadingError { msg: err.to_string() }
    }
}

impl From<std::str::Utf8Error> for BMError {
    fn from(err: std::str::Utf8Error) -> Self {
        BMError::LoadingError { msg: err.to_string() }
    }
}

impl fmt::Display for BMError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      BMError::LoadingError { msg } => write!(f, "LoadingError: {}", msg),
    }
  }
}


