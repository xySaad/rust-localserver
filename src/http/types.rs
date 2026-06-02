use std::result;

#[derive(Debug)]
pub enum Error {
    BadRequest = 400,
    NotFound = 404,
    InternalError = 500,
}

pub type Result<T = ()> = result::Result<T, Error>;
