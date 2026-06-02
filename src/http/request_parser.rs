use std::{io::Read, net::TcpStream};

use crate::http::{
    self,
    Error::BadRequest,
    request_parser::ParseState::{HeaderLine, StartLine},
};

#[derive(Debug)]
enum ParseState {
    StartLine,
    HeaderLine,
}

pub struct RequestParser {
    state: ParseState,
    request_buf: Vec<u8>,
}

impl RequestParser {
    pub fn new() -> Self {
        return RequestParser {
            state: StartLine,
            request_buf: Vec::new(),
        };
    }
    fn parse_start_line(self: &mut Self, chunk: &[u8]) -> http::Result {
        let mut last_eol = 0;

        for (i, b) in chunk.iter().enumerate() {
            if *b == b'\n' {
                //append newly written data until new line found
                self.buffer_request_chunk(&chunk[last_eol..i]);
                let line = &self.request_buf;

                //parse line and mark last eol
                let line_str = String::from_utf8_lossy(line);
                println!("start line -> {line_str}");

                let mut parts = line.split(|b| *b == b' ');
                let method = parts.next().ok_or(BadRequest)?;
                let request_target = parts.next().ok_or(BadRequest)?;
                let protocol = parts.next().ok_or(BadRequest)?;
                // HTTP/1.1

                last_eol = i + 1;

                //clear parsed line
                self.request_buf.clear();
                self.state = HeaderLine;
                break;
            }
        }

        // buffer unparsed chunk
        self.buffer_request_chunk(&chunk[last_eol..]);
        return Ok(());
    }

    fn parse_header_line(self: &mut Self, chunk: &[u8]) -> http::Result {
        let mut last_eol = 0;

        for (i, b) in chunk.iter().enumerate() {
            if *b == b'\n' {
                //append newly written data until new line found
                self.buffer_request_chunk(&chunk[last_eol..i]);
                let line = &self.request_buf;
                //parse line and mark last eol
                let line_str = String::from_utf8_lossy(line);
                println!("header line -> {line_str}");
                last_eol = i + 1;

                //clear parsed line
                self.request_buf.clear();
            }
        }

        // buffer unparsed chunk
        self.buffer_request_chunk(&chunk[last_eol..]);
        return Ok(());
    }

    fn buffer_request_chunk(self: &mut Self, chunk: &[u8]) {
        let chunk_vectored = &mut Vec::from(chunk);
        self.request_buf.append(chunk_vectored);
    }

    pub fn parse(self: &mut Self, stream: &mut TcpStream) -> http::Result {
        let read_buf = &mut [0; 500];

        if self.request_buf.len() > 0 {
            let chunk = &self.request_buf.clone();
            self.request_buf.clear();
            self.parse_chunk(chunk)?;
        } else {
            if let Ok(n) = stream.read(read_buf)
                && n > 0
            {
                let chunk = &read_buf[..n];
                self.parse_chunk(chunk)?;
            }
        }

        return Ok(());
    }

    pub fn parse_chunk(self: &mut Self, chunk: &[u8]) -> http::Result {
        let Self { state, .. } = self;

        match state {
            StartLine => self.parse_start_line(chunk)?,
            HeaderLine => self.parse_header_line(chunk)?,
        };

        return Ok(());
    }
}
