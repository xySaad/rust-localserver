use core::fmt;
use std::result;

#[derive(Debug, Clone, Copy)]
pub enum Status {
    OK = 200,
    BadRequest = 400,
    Unauthorized = 401,
    NotFound = 404,
    MisdirectedRequest = 421,
    InternalError = 500,
    MovedPermanently = 301,
}
impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as isize)
    }
}
pub type Result<T = ()> = result::Result<T, Status>;
