use std::collections::HashMap;
use std::fmt;

use crate::future::AsyncRead;
use crate::http::{Headers, Request};

/// Parses the raw value of a `Cookie` request header, e.g. `"a=1; b=2"`,
/// into a name -> value map.
pub fn parse_cookies(header_value: &str) -> HashMap<String, String> {
    header_value
        .split(';')
        .filter_map(|pair| pair.trim().split_once('='))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .collect()
}

impl<AR: AsyncRead> Request<AR> {
    /// Parses every cookie the client sent in the `Cookie` header.
    pub fn cookies(&self) -> HashMap<String, String> {
        self.header("Cookie").map(parse_cookies).unwrap_or_default()
    }

    /// Returns the value of a single cookie, if the client sent one by that name.
    pub fn cookie(&self, name: &str) -> Option<String> {
        self.cookies().remove(name)
    }
}

/// Controls whether a cookie is also sent along with cross-site requests.
/// See rfc6265bis for the meaning of each value.
#[derive(Debug)]
pub enum SameSite {
    Strict,
    Lax,
    None,
}

/// A cookie to be sent to the client through a `Set-Cookie` response header.
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub path: Option<String>,
    pub max_age: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<SameSite>,
}

impl Cookie {
    pub fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: value.to_owned(),
            path: None,
            max_age: None,
            http_only: false,
            secure: false,
            same_site: None,
        }
    }

    /// restricts the cookie to a path prefix, e.g. "/"
    pub fn path(mut self, path: &str) -> Self {
        self.path = Some(path.to_owned());
        self
    }

    /// how many seconds the cookie should live for. use 0 to delete it immediately.
    pub fn max_age(mut self, seconds: i64) -> Self {
        self.max_age = Some(seconds);
        self
    }

    /// hides the cookie from JavaScript (`document.cookie`)
    pub fn http_only(mut self, yes: bool) -> Self {
        self.http_only = yes;
        self
    }

    /// only send the cookie back over HTTPS
    pub fn secure(mut self, yes: bool) -> Self {
        self.secure = yes;
        self
    }

    pub fn same_site(mut self, same_site: SameSite) -> Self {
        self.same_site = Some(same_site);
        self
    }

    /// builds a cookie that instructs the client to erase a previously set one.
    pub fn remove(name: &str) -> Self {
        Self::new(name, "").path("/").max_age(0)
    }
}

impl fmt::Display for Cookie {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)?;

        if let Some(path) = &self.path {
            write!(f, "; Path={path}")?;
        }
        if let Some(max_age) = self.max_age {
            write!(f, "; Max-Age={max_age}")?;
        }
        if let Some(same_site) = &self.same_site {
            write!(f, "; SameSite={same_site:?}")?;
        }
        if self.http_only {
            write!(f, "; HttpOnly")?;
        }
        if self.secure {
            write!(f, "; Secure")?;
        }

        Ok(())
    }
}

pub trait SetCookie {
    fn set_cookie(&mut self, cookie: Cookie);
}

impl SetCookie for Headers {
    /// Adds a `Set-Cookie` entry to headers.
    fn set_cookie(&mut self, cookie: Cookie) {
        self.entry("Set-Cookie".to_owned())
            .or_insert_with(Vec::new)
            .push(cookie.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_multiple_cookies() {
        let cookies = parse_cookies("session_id=abc123; theme=dark");
        assert_eq!(cookies.get("session_id"), Some(&"abc123".to_owned()));
        assert_eq!(cookies.get("theme"), Some(&"dark".to_owned()));
    }

    #[test]
    fn ignores_malformed_pairs() {
        let cookies = parse_cookies("valid=1; nonsense; also_valid=2");
        assert_eq!(cookies.len(), 2);
    }

    #[test]
    fn builds_set_cookie_header() {
        let cookie = Cookie::new("session_id", "abc123")
            .path("/")
            .http_only(true)
            .max_age(3600);
        assert_eq!(cookie.to_string(), "session_id=abc123; Path=/; Max-Age=3600; HttpOnly");
    }

    #[test]
    fn remove_expires_immediately() {
        let cookie = Cookie::remove("session_id");
        assert_eq!(cookie.to_string(), "session_id=; Path=/; Max-Age=0");
    }

    #[test]
    fn set_cookie_appends_instead_of_overwriting() {
        let mut headers = Headers::new();
        headers.set_cookie(Cookie::new("a", "1"));
        headers.set_cookie(Cookie::new("b", "2"));
        assert_eq!(headers.get("Set-Cookie").map(Vec::len), Some(2));
    }
}
