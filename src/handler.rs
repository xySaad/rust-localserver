use crate::{
    cgi_executor::CGIExecutor,
    file_server::{FileServer, HTTPBodyReader},
    future::{AsyncTcpStream, AsyncWrite, copy},
    http::{
        self, Headers, Request,
        Status::{self},
    },
    parser::ServerConfig,
};
use std::{format, path::Path};

async fn handle_file_serving(
    req: &mut Request<&mut AsyncTcpStream>,
    root: &str,
    index: &str,
    list_directory: bool,
    error_pages_dir: &str,
) -> http::Result<(Status, Headers, Option<HTTPBodyReader>)> {
    let file_server = FileServer::new(root, index, list_directory);
    match file_server.serve(&req).await {
        ok @ Ok(_) => return ok,
        Err(status) => {
            //modify the target to the error page file
            req.meta.path = format!("/{error_pages_dir}/{status}.html");
            return file_server.serve(&req).await.map_err(|_| status);
        }
    }
}

async fn handle_request<'r, 't>(
    req: &'r mut Request<&'t mut AsyncTcpStream>,
    server_config_list: &'t Vec<(String, ServerConfig)>,
) -> http::Result<Option<(Status, Headers, Option<HTTPBodyReader>)>> {
    let req_host = req.header("Host").map(|s| s.to_owned()).ok_or(Status::BadRequest)?;
    let (_name, cfg) = server_config_list
        .into_iter()
        .find(|(_name, cfg)| cfg.host.contains(&req_host))
        .ok_or(Status::MisdirectedRequest)?;

    let ServerConfig {
        root,
        list_directory,
        index,
        error_pages_dir,
        client_body_size_limit,
        cgi_root,
        ..
    } = cfg;

    let root_path = Path::new(root);
    let absolute_cgi_root = root_path
        .join(cgi_root.trim_start_matches('/'))
        .canonicalize()
        .map_err(|_| Status::BadRequest)?;
    let user_path = Path::new(&req.meta.path);
    let user_path = user_path.strip_prefix("/").map_err(|_| Status::BadRequest)?;
    let cgi_request_path = root_path.join(user_path).canonicalize().map_err(|_| Status::NotFound)?;

    if cgi_request_path.starts_with(absolute_cgi_root) {
        CGIExecutor::exec(
            req,
            cgi_request_path.to_string_lossy().to_string(),
            client_body_size_limit,
        )
        .await?;
        return Ok(None);
    } else {
        let response_parts = handle_file_serving(req, root, index, *list_directory, error_pages_dir).await?;
        return Ok(Some(response_parts));
    };
}

pub async fn server_handler<'t>(
    mut req: Request<&mut AsyncTcpStream>,
    server_config_list: &'t Vec<(String, ServerConfig)>,
) {
    let result = handle_request(&mut req, server_config_list).await; // borrow ends here
    match result {
        Ok(None) => (), //the handler already did write the response
        Ok(Some((status, headers, body))) => {
            let mut wr = req.response().status(status).await.headers(headers).await;
            if let Some(mut reader) = body {
                let _ = copy(&mut reader, &mut wr).await;
            }
        }
        Err(status) => {
            _ = req
                .response()
                .status(status)
                .await
                .headers(Headers::new())
                .await
                .write(format!("{0} {0:?}", status).as_bytes())
                .await
        }
    };
}
