use std::{
    ffi::OsString,
    fs::{self, File, OpenOptions},
    ops::Add,
    path::Path,
};

use crate::{
    future::{AsyncWrite, ReadFuture},
    http::{self, BodyWriter, Headers, Request, Response, Status},
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

    async fn get_file(&self, req: &Request<'_>) -> http::Result<FileResult> {
        let root = Path::new(self.root).canonicalize().map_err(|_| Status::BadRequest)?;
        let user_path = Path::new(&req.meta.request_target);
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

        return Err(Status::NotFound);
    }

    pub async fn serve(&self, req: Request<'_>) {
        match self.get_file(&req).await {
            Ok(FileResult::Dir(_)) if !req.meta.request_target.ends_with('/') => {
                let location = format!("{}/", req.meta.request_target);
                let mut headers = Headers::new();
                headers.insert("Location".to_owned(), vec![location]);
                _ = req
                    .response()
                    .status(Status::MovedPermanently)
                    .await
                    .headers(headers)
                    .await;
            }
            Ok(ref mut res) => self.serve_file(req.response(), res).await,
            Err(status) => {
                let body = format!("{status} {status:?}");
                _ = req
                    .response()
                    .status(status)
                    .await
                    .headers(Headers::new())
                    .await
                    .write(body.as_bytes())
                    .await;
            }
        }
    }

    async fn serve_file(&self, resp: Response<'_>, result: &mut FileResult) {
        match result {
            FileResult::File(file) => {
                let mut wr = resp.status(Status::OK).await.headers(Headers::new()).await;
                self.write_file(&mut wr, file).await;
            }
            FileResult::Dir(items) => {
                let mut headers = Headers::new();
                headers.insert("Content-Type".to_owned(), vec!["text/html".to_owned()]);
                let mut wr = resp.status(Status::OK).await.headers(headers).await;

                let post = String::new()
                    .add("<!doctype html>\n")
                    .add("<meta name=\"viewport\" content=\"width=device-width\">\n")
                    .add("<pre>");

                _ = wr.write(post.as_bytes()).await;
                for item in items {
                    let file_name = item.to_string_lossy().to_string();
                    let line = format!("<a href=\"{file_name}\">{file_name}</a>\n");
                    _ = wr.write(line.as_bytes()).await;
                }

                _ = wr.write("</pre>\n".as_bytes()).await;
            }
        }
    }

    async fn write_file<W: AsyncWrite>(&self, wr: &mut BodyWriter<'_, W>, file: &mut File) {
        let buf = &mut [0; 512];

        loop {
            let n = ReadFuture { buf, reader: file }.await.unwrap_or(0);
            if n == 0 {
                break;
            }
            _ = wr.write(buf).await;
        }
    }
}
