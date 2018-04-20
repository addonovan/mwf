use std::error::Error as StdError;
use std::io::Error as IoError;
use std::fmt;
use std::result;

/// A result type which used [mwf::Error].
pub type Result<T> = result::Result<T, Error>;

/// A generic error type for mwf.
///
/// If you think I should add another error type, please submit a pull
/// request.
#[derive(Debug)]
pub enum Error
{
    Io(IoError),
    Other(Box<StdError + Send>),
}

impl From<IoError> for Error
{
    fn from(error: IoError) -> Self
    {
        Error::Io(error)
    }
}

impl StdError for Error
{
    fn description(&self) -> &str
    {
        match self {
            &Error::Io(ref cause) => cause.description(),
            &Error::Other(ref cause) => cause.description(),
        }
    }

    fn cause(&self) -> Option<&StdError>
    {
        match self {
            &Error::Io(ref cause) => cause.cause(),
            &Error::Other(ref cause) => cause.cause(),
        }
    }
}

impl fmt::Display for Error
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self {
            &Error::Io(ref cause) => cause.fmt(f),
            &Error::Other(ref cause) => cause.fmt(f),
        }
    }
}

