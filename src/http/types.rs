use core::fmt;
use std::result;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Continue = 100,
    SwitchingProtocols = 101,

    OK = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInformation = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,

    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    /// Deprecated per RFC 9110 §15.4.6
    UseProxy = 305,
    /// Reserved; no longer used per RFC 9110 §15.4.8
    SwitchProxy = 306,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthenticationRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    ContentTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    /// Reserved; not intended for use per RFC 9110 §15.5.19
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableContent = 422,
    UpgradeRequired = 426,

    InternalError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as isize)
    }
}

pub type Result<T = ()> = result::Result<T, Status>;
