use std::{
    cell::RefCell,
    collections::HashMap,
    os::fd::AsRawFd,
    process::{Command, Stdio},
};

use bytesize::ByteSize;

use crate::{
    future::{AsyncTcpStream, Pool, ReadFuture, Task, WriteFuture},
    http::{self, Headers, Request, RequestLine, Status},
};

pub struct CGIExecutor {}

impl CGIExecutor {
    pub async fn exec<'t>(
        req: &mut Request<&'t mut AsyncTcpStream>,
        cgi_file_path: String,
        client_body_size_limit: &'t ByteSize,
    ) -> http::Result<()> {
        let env_vars = build_cgi_env(&req.meta, &req.headers);
        let child = Command::new(cgi_file_path)
            .envs(env_vars)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Status::InternalError)?;

        let (mut stdin, mut stdout) = match (child.stdin, child.stdout) {
            (Some(stdin), Some(stdout)) => (stdin, stdout),
            _ => return Err(Status::InternalError),
        };
        set_nonblocking(&stdin)?;
        set_nonblocking(&stdout)?;

        let stdout = &mut stdout;
        let req = RefCell::new(req);
        let body_size_limit = client_body_size_limit.as_u64() as usize;

        let request_reader = async {
            let mut sent: usize = 0;
            loop {
                let mut buf = [0; 1024];
                match req.borrow_mut().read_body(&mut buf).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        sent += n;
                        if sent > body_size_limit {
                            break;
                        }

                        let write_future = WriteFuture {
                            buf: &mut buf[..n],
                            writer: &mut stdin,
                        };
                        if write_future.await.is_err() {
                            break;
                        }
                    }
                }
            }
            drop(stdin);
        };

        let response_writer = async {
            let mut buf_out: Vec<u8> = Vec::new();
            let (header_end, body_start) = loop {
                if let Some(split) = find_header_terminator(&buf_out) {
                    break split;
                }

                let mut buf = [0; 1024];
                let read_future = ReadFuture {
                    buf: &mut buf,
                    reader: stdout,
                };
                match read_future.await {
                    Ok(0) | Err(_) => break (buf_out.len(), buf_out.len()),
                    Ok(n) => buf_out.extend_from_slice(&buf[..n]),
                }
            };

            let cgi_headers = parse_cgi_headers(&buf_out[..header_end]);
            let (status_code, reason) = resolve_status(&cgi_headers);

            let mut out = format!("{} {status_code} {reason}\r\n", req.borrow().meta.protocol);
            for (key, value) in &cgi_headers {
                if key.eq_ignore_ascii_case("status") {
                    continue;
                }
                out.push_str(key);
                out.push_str(": ");
                out.push_str(value);
                out.push_str("\r\n");
            }
            out.push_str("\r\n");

            if req.borrow_mut().write_raw_response(out.as_bytes()).await.is_err() {
                return;
            }

            if body_start < buf_out.len()
                && req
                    .borrow_mut()
                    .write_raw_response(&buf_out[body_start..])
                    .await
                    .is_err()
            {
                return;
            }

            loop {
                let mut buf = [0; 1024];
                let read_future = ReadFuture {
                    buf: &mut buf,
                    reader: stdout,
                };

                match read_future.await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        if req.borrow_mut().write_raw_response(&buf[..n]).await.is_err() {
                            break;
                        }
                    }
                }
            }
        };

        let mut pool = Pool::new();
        pool.add_task(Task::new(request_reader));
        pool.add_task(Task::new(response_writer));
        pool.await_all().await;
        Ok(())
    }
}

/// Finds the blank line that separates the CGI header block from the body.
/// Returns `(header_end, body_start)`: bytes before `header_end` are
/// headers, bytes from `body_start` onward are body.
fn find_header_terminator(buf: &[u8]) -> Option<(usize, usize)> {
    buf.windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| (p, p + 4))
        .or_else(|| buf.windows(2).position(|w| w == b"\n\n").map(|p| (p, p + 2)))
}

fn header_ci<'a>(headers: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

/// A `Status` header sets the status explicitly; otherwise a `Location`
/// header implies a redirect; otherwise default to 200.
fn resolve_status(headers: &HashMap<String, String>) -> (u16, String) {
    if let Some(v) = header_ci(headers, "status") {
        let mut parts = v.splitn(2, ' ');
        if let Some(code) = parts.next().and_then(|c| c.parse::<u16>().ok()) {
            let reason = parts.next().unwrap_or("").trim().to_string();
            return (code, reason);
        }
    }

    if header_ci(headers, "location").is_some() {
        return (302, "Found".to_string());
    }

    (200, "OK".to_string())
}

fn set_nonblocking<H: AsRawFd>(h: &H) -> http::Result<()> {
    unsafe {
        let fd = h.as_raw_fd();
        let flags = libc::fcntl(fd, libc::F_GETFL, 0);
        if flags < 0 {
            return Err(Status::InternalError);
        }
        if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
            return Err(Status::InternalError);
        }
        Ok(())
    }
}

fn build_cgi_env(meta: &RequestLine, headers: &Headers) -> HashMap<String, String> {
    let mut env = HashMap::new();
    for (key, values) in headers {
        let upper = key.to_ascii_uppercase().replace('-', "_");
        values
            .last()
            .and_then(|v| env.insert(format!("HTTP_{upper}"), v.clone()));
    }

    headers
        .get("Content-Type")
        .and_then(|v| v.last())
        .and_then(|v| env.insert("CONTENT_TYPE".to_string(), v.clone()));
    headers
        .get("Content-Length")
        .and_then(|v| v.last())
        .and_then(|v| env.insert("CONTENT_LENGTH".to_string(), v.clone()));
    env.insert("REQUEST_METHOD".to_string(), meta.method.clone());
    env.insert("SCRIPT_NAME".to_string(), meta.path.clone());
    env.insert("QUERY_STRING".to_string(), meta.query.clone());

    return env;
}

fn parse_cgi_headers(buf: &[u8]) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    for line in String::from_utf8_lossy(buf).split('\n') {
        let line = line.trim_end_matches('\r');
        line.split_once(':')
            .and_then(|(k, v)| headers.insert(k.trim().to_string(), v.trim().to_string()));
    }
    headers
}
