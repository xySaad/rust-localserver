use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io,
    ops::Add,
    path::Path,
};

use crate::{
    future::{AsyncRead, AsyncTcpStream, ReadFuture},
    http::{self, Headers, Request, Status},
};

pub struct FileServer<'t> {
    pub root: &'t str,
    pub index: &'t str,
    pub list_directory: bool,
}

#[derive(Debug)]
enum FileResult {
    File(File),
    Dir(Vec<OsString>),
}

impl<'t> FileServer<'t> {
    pub fn new(root: &'t str, index: &'t str, list_directory: bool) -> Self {
        Self {
            root,
            index,
            list_directory,
        }
    }

    async fn get_file(&self, req: &Request<&mut AsyncTcpStream>) -> http::Result<FileResult> {
        let root = &Path::new(self.root).canonicalize().map_err(|_| Status::BadRequest)?;
        let user_path = Path::new(&req.meta.path);
        let user_path = user_path.strip_prefix("/").map_err(|_| Status::BadRequest)?;
        let path = root.join(user_path).canonicalize().map_err(|_| Status::NotFound)?;

        if !path.starts_with(root) || !path.exists() {
            return Err(Status::NotFound);
        }

        if !path.is_dir() {
            let file = OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|_| Status::InternalError)?;
            return Ok(FileResult::File(file));
        }

        if self.list_directory {
            let dir = fs::read_dir(path).map_err(|_| Status::InternalError)?;
            let mut files = vec![];
            for entry in dir {
                let entry = entry.map_err(|_| Status::InternalError)?;
                let file_name = entry.file_name();
                files.push(file_name);
            }
            return Ok(FileResult::Dir(files));
        }

        if !self.index.is_empty() {
            let path = root.join(self.index).canonicalize().map_err(|_| Status::NotFound)?;
            if !path.starts_with(root) || !path.exists() {
                return Err(Status::NotFound);
            }
            let file = OpenOptions::new()
                .read(true)
                .open(path)
                .map_err(|_| Status::InternalError)?;
            return Ok(FileResult::File(file));
        }
        return Err(Status::NotFound);
    }

    pub async fn serve(
        &self,
        req: &Request<&mut AsyncTcpStream>,
    ) -> http::Result<(Status, Headers, Option<HTTPBodyReader>)> {
        if req.meta.method != "GET" {
            return Err(Status::MethodNotAllowed);
        }

        match self.get_file(&req).await? {
            FileResult::Dir(_) if !req.meta.path.ends_with('/') => {
                let location = format!("{}/", req.meta.path);
                let mut headers = Headers::new();
                headers.insert("Location".to_owned(), vec![location]);
                return Ok((Status::MovedPermanently, headers, None));
            }
            file_result => {
                let http_result = match file_result {
                    FileResult::File(file) => (
                        Status::OK,
                        Headers::new(),
                        Some(HTTPBodyReader::File(HTTPFileReader { file })),
                    ),
                    FileResult::Dir(os_strings) => {
                        let mut headers = Headers::new();
                        headers.insert("Content-Type".to_owned(), vec!["text/html".to_owned()]);
                        (
                            Status::OK,
                            headers,
                            Some(HTTPBodyReader::Dir(HTTPDirReader::new(os_strings))),
                        )
                    }
                };
                return Ok(http_result);
            }
        }
    }
}

pub enum HTTPBodyReader {
    File(HTTPFileReader),
    Dir(HTTPDirReader),
}

impl AsyncRead for HTTPBodyReader {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            HTTPBodyReader::File(r) => r.read(buf).await,
            HTTPBodyReader::Dir(r) => r.read(buf).await,
        }
    }
}

pub struct HTTPFileReader {
    file: File,
}
impl AsyncRead for HTTPFileReader {
    async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        return ReadFuture {
            buf,
            reader: &mut self.file,
        }
        .await;
    }
}
pub struct HTTPDirReader {
    content: io::Cursor<Vec<u8>>,
}

impl HTTPDirReader {
    pub fn new(entries: Vec<OsString>) -> Self {
        let mut body = String::new()
            .add("<!doctype html>\n")
            .add("<meta name=\"viewport\" content=\"width=device-width\">\n")
            .add("<pre>\n");

        for item in &entries {
            let file_name = item.to_string_lossy().to_string();
            body.push_str(&format!("<a href=\"{file_name}\">{file_name}</a>\n"));
        }

        body.push_str("</pre>\n");

        Self {
            content: io::Cursor::new(body.into_bytes()),
        }
    }
}

impl AsyncRead for HTTPDirReader {
    async fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        use std::io::Read;
        self.content.read(buf)
    }
}
