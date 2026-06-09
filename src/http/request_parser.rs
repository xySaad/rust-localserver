use crate::{
    future::{AsyncBufferReader, AsyncRead, BufferRead},
    http::{
        self, Headers, RequestLine,
        Status::{BadRequest, InternalError},
    },
};
pub struct RequestParser<AR: AsyncRead> {
    reader: AsyncBufferReader<AR>,
}

impl<AR: AsyncRead> From<AR> for RequestParser<AR> {
    fn from(reader: AR) -> Self {
        let reader = AsyncBufferReader::from(reader);
        return Self { reader };
    }
}

fn clean_cr(value: &str) -> String {
    value.strip_suffix('\r').unwrap_or(value).trim().to_owned()
}
fn lossy_string_or_empty(v: &[u8]) -> String {
    String::from_utf8_lossy(v).to_string()
}
impl<AR: AsyncRead> RequestParser<AR> {
    pub async fn parse_request_line(self: &mut Self) -> http::Result<RequestLine> {
        let line = &mut Vec::new();
        let _n = self.reader.read_until(b'\n', line).await.map_err(|_e| InternalError)?;

        //TODO: if n is 0 return an error like 'connection closed'
        let mut parts = line.split(|b| *b == b' ');
        //TODO: add check for allowed methods
        let method = parts.next().ok_or(BadRequest)?;
        //TODO: add check for request target max_length
        let request_target = parts.next().ok_or(BadRequest)?;
        let protocol = parts.next().ok_or(BadRequest)?;
        if protocol != "HTTP/1.1\r".as_bytes() {
            return Err(BadRequest);
        }

        let method = String::from_utf8_lossy(method).to_string();
        let request_target = String::from_utf8_lossy(request_target).to_string();
        let protocol = String::from_utf8_lossy(protocol).to_string();
        let request_meta = RequestLine {
            method,
            request_target,
            protocol,
        };
        return Ok(request_meta);
    }

    pub async fn parse_headers(self: &mut Self) -> http::Result<Headers> {
        let mut headers = Headers::new();

        loop {
            let line = &mut Vec::new();
            let _n = self.reader.read_until(b'\n', line).await.map_err(|_e| InternalError)?;
            if line.len() == 1 && line[0] == b'\r' {
                break;
            }
            let separator = line.iter().position(|b| *b == b':').ok_or(BadRequest)?;
            let header_name = lossy_string_or_empty(&line[..separator]);
            let header_value = lossy_string_or_empty(&line[separator + 1..]);
            let clean_header_name = header_name.trim().to_owned();
            let clean_header_value = clean_cr(&header_value);

            headers
                .entry(clean_header_name)
                .or_insert(vec![])
                .push(clean_header_value);
        }

        return Ok(headers);
    }

    pub async fn parse(self: &mut Self) -> http::Result<(RequestLine, Headers)> {
        let start_line = self.parse_request_line().await?;

        let headers = self.parse_headers().await?;
        return Ok((start_line, headers));
    }
}
