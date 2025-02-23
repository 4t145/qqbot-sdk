use std::borrow::Cow;

use crate::http::api::ResponseFail;
pub type Result<T> = std::result::Result<T, Error>;
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    context: Cow<'static, str>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.kind)
    }
}

impl Error {
    pub fn new(kind: ErrorKind, context: impl Into<Cow<'static, str>>) -> Self {
        Self {
            kind,
            context: context.into(),
        }
    }
    pub const fn context<K: Into<ErrorKind>>(
        context: impl Into<Cow<'static, str>>,
    ) -> impl FnOnce(K) -> Error {
        move |kind| Error::new(kind.into(), context)
    }
    pub fn unexpected(context: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::Unexpected, context)
    }
    pub fn timeout(context: impl Into<Cow<'static, str>>) -> Self {
        Self::new(ErrorKind::Timeout, context)
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for ErrorKind {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<ResponseFail> for ErrorKind {
    fn from(err: ResponseFail) -> Self {
        Self::ResponseFail(err)
    }
}
#[derive(Debug)]
pub enum ErrorKind {
    SerdeJson(serde_json::Error),
    Io(std::io::Error),
    Reqwest(reqwest::Error),
    ResponseFail(ResponseFail),
    Unexpected,
    Timeout,
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unexpected => write!(f, "unexpected error"),
            Self::SerdeJson(err) => write!(f, "serde_json error: {}", err),
            Self::Io(err) => write!(f, "io error: {}", err),
            Self::Reqwest(err) => write!(f, "reqwest error: {}", err),
            Self::Timeout => write!(f, "timeout"),
            Self::ResponseFail(err) => write!(f, "response fail: {}({})", err.message, err.code),
        }
    }
}
