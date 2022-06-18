use thiserror;
use thiserror::Error;
use std::path::Display;


#[derive(Debug, thiserror::Error)]
pub enum NfError {
    #[error("io error: `{0}`")]

    IoError(String),

    #[error("the key `{0}` is no exists.")]
    KeyError(String),

    #[error("the variable `{0}` is None")]
    NoneError(String),

    #[error("error: `{0}`")]
    E(String),

    #[error("convert error: `{0}`")]
    ConvertError(String),

    #[error("anyhow error")]
    Other(#[from] anyhow::Error)
}

impl NfError {
    pub fn description(&self) -> String {
        use NfError::*;
        let e = match self {
            Other(e) => anyhow_error_to_chain(e),
            IoError(e) => e.to_string(),
            KeyError(e) => e.to_string(),
            NoneError(e) => e.to_string(),
            E(e) => e.to_string(),
            ConvertError(e) => e.to_string(),
            _ => "".to_string()
        };
        e.to_string()
    }

    pub fn to_string(&self) -> String {
        self.description()
    }
}


#[repr(i32)]
pub enum NfErrorCode {
    Success = 0,
    Fail = -1,
}

pub type NfResult<T> = Result<T, NfError>;


pub fn anyhow_error_to_chain(e: &anyhow::Error) -> String {
    let mut err = "".to_string();
    e.chain().next()
        .iter()
        .enumerate()
        .for_each(|(index, e)| err = format!("{}\n{} - {}", err, index, e));
    err
}