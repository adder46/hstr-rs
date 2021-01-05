use errno::errno;
use std::fmt;

#[derive(Debug)]
/// Represents an error that has occurred within the History API.
pub struct HistoryError {
    desc: String,
    detail: String,
}

impl HistoryError {
    /// Create a HistoryError struct from the given description and detail.
    pub fn new<T>(desc: &str, detail: T) -> HistoryError
    where
        T: fmt::Display,
    {
        HistoryError {
            desc: String::from(desc),
            detail: format!("{}", detail),
        }
    }
}

/// Implemented as 'self.desc: self.detail'.
impl fmt::Display for HistoryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.desc, self.detail)
    }
}

impl From<::std::ffi::NulError> for HistoryError {
    fn from(e: ::std::ffi::NulError) -> HistoryError {
        HistoryError::new("NulError", e)
    }
}

impl From<::std::str::Utf8Error> for HistoryError {
    fn from(e: ::std::str::Utf8Error) -> HistoryError {
        HistoryError::new("FromUtf8Error", e)
    }
}

impl From<::std::num::ParseIntError> for HistoryError {
    fn from(e: ::std::num::ParseIntError) -> HistoryError {
        HistoryError::new("ParseIntError", e)
    }
}

pub fn gen_result(res: i32) -> Result<i32, HistoryError> {
    if res == 0 {
        Ok(res)
    } else {
        let e = errno();
        let code = e.0 as i32;
        let out = format!("Error {}: {}", code, e);
        Err(HistoryError::new("History Error", &out[..]))
    }
}
