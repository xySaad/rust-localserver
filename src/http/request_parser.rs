use std::{io::Read, mem, net::TcpStream};

use crate::http::{
    self,
    Error::BadRequest,
    RequestHeaders, RequestMeta,
    request_parser::ParseState::{HeaderLine, StartLine, Transitioning},
};

#[derive(Default)]
enum ParseState {
    StartLine(StartLineParser),
    HeaderLine(HeaderLineParser),
    #[default]
    Transitioning,
}

pub struct StartLineParser {
    buffer: Vec<u8>,
}

impl StartLineParser {
    fn new() -> Self {
        return StartLineParser { buffer: vec![] };
    }

    fn parse(self: &mut Self, chunk: &[u8]) -> http::Result<Option<RequestMeta>> {
        let i = if let Some(i) = chunk.iter().position(|b| *b == b'\n') {
            i
        } else {
            // buffer unparsed chunk
            self.buffer.extend_from_slice(&chunk);
            return Ok(None);
        };

        //append newly written data until new line found
        self.buffer.extend_from_slice(&chunk[..i]);
        let line = &self.buffer;

        //parse line
        let mut parts = line.split(|b| *b == b' ');
        let method = parts.next().ok_or(BadRequest)?;
        let request_target = parts.next().ok_or(BadRequest)?;
        let protocol = parts.next().ok_or(BadRequest)?;
        if protocol != "HTTP/1.1\r".as_bytes() {
            return Err(BadRequest);
        }
        let method = String::from_utf8_lossy(method).to_string();
        let request_target = String::from_utf8_lossy(request_target).to_string();
        let protocol = String::from_utf8_lossy(protocol).to_string();
        let request_meta = RequestMeta {
            method,
            request_target,
            protocol,
        };

        //clear parsed line
        self.buffer.clear();

        // buffer unparsed chunk
        self.buffer.extend_from_slice(&chunk[i + 1..]);
        return Ok(Some(request_meta));
    }
}

pub struct HeaderLineParser {
    request_meta: RequestMeta,
    buffer: Vec<u8>,
    headers: RequestHeaders,
}
impl HeaderLineParser {
    fn new(request_meta: RequestMeta) -> Self {
        return Self {
            request_meta,
            buffer: vec![],
            headers: RequestHeaders::new(),
        };
    }

    fn parse(self: &mut Self, chunk: &[u8]) -> http::Result<bool> {
        let mut last_eol = 0;

        for (i, b) in chunk.iter().enumerate() {
            if *b == b'\n' {
                let current_eol = last_eol;
                last_eol = i + 1;

                // break on \r
                if i - current_eol == 1 && chunk[i - 1] == b'\r' {
                    // buffer unparsed chunk
                    self.buffer.extend_from_slice(&chunk[last_eol..]);
                    return Ok(true);
                };

                //append newly written data until new line found
                self.buffer.extend_from_slice(&chunk[current_eol..i]);
                let line = &self.buffer;
                //parse line and mark last eol
                let separator = line.iter().position(|b| *b == b':').ok_or(BadRequest)?;
                let header_name = String::from_utf8_lossy(&line[..separator]).to_string();
                let header_value = String::from_utf8_lossy(&line[separator..]).to_string();
                self.headers.entry(header_name).or_insert(vec![]).push(header_value);
                //clear parsed line
                self.buffer.clear();
            }
        }

        // buffer unparsed chunk
        self.buffer.extend_from_slice(&chunk[last_eol..]);
        return Ok(false);
    }
}
pub type ParsedRequest = (RequestMeta, RequestHeaders);
pub struct RequestParser {
    state: ParseState,
    request_buf: Vec<u8>,
}

impl RequestParser {
    pub fn new() -> Self {
        return RequestParser {
            state: StartLine(StartLineParser::new()),
            request_buf: Vec::new(),
        };
    }

    pub fn parse(self: &mut Self, stream: &mut TcpStream) -> http::Result<Option<ParsedRequest>> {
        let read_buf = &mut [0; 500];

        if self.request_buf.len() > 0 {
            let chunk = &self.request_buf.clone();
            self.request_buf.clear();
            return self.parse_chunk(chunk);
        } else {
            if let Ok(n) = stream.read(read_buf)
                && n > 0
            {
                let chunk = &read_buf[..n];
                return self.parse_chunk(chunk);
            }
        }

        return Ok(None);
    }

    pub fn parse_chunk(self: &mut Self, chunk: &[u8]) -> http::Result<Option<ParsedRequest>> {
        match mem::take(&mut self.state) {
            Transitioning => unreachable!(),
            StartLine(mut start_line_parser) => {
                if let Some(request_meta) = start_line_parser.parse(chunk)? {
                    self.request_buf.extend_from_slice(&start_line_parser.buffer);
                    self.state = HeaderLine(HeaderLineParser::new(request_meta));
                };
            }
            HeaderLine(mut header_line_parser) => {
                let end_of_headers = header_line_parser.parse(chunk)?;
                if end_of_headers {
                    return Ok(Some((header_line_parser.request_meta, header_line_parser.headers)));
                }
            }
        };

        return Ok(None);
    }
}
