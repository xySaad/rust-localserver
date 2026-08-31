use crate::{
    file_server::{FileServer, HTTPFileReader},
    future::{AsyncTcpStream, AsyncWrite},
    http::{
        self, Headers, Request,
        Status::{self},
    },
    parser::ServerConfig,
};
use std::format;

async fn serve_file<'t>(
    req: &mut Request<&mut AsyncTcpStream>,
    server_config_list: &'t Vec<(String, ServerConfig)>,
) -> http::Result<(Status, Headers, Option<HTTPFileReader>)> {
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
        ..
    } = cfg;

    let file_server = FileServer::new(root, index, *list_directory);
    match file_server.serve(&req).await {
        ok @ Ok(_) => return ok,
        Err(status) => {
            //modify the target to the error page file
            req.meta.request_target = format!("/{error_pages_dir}/{status}.html");
            return file_server.serve(&req).await.map_err(|_| status);
        }
    }
}

pub async fn server_handler<'t>(
    mut req: Request<&mut AsyncTcpStream>,
    server_config_list: &'t Vec<(String, ServerConfig)>,
) {
    match serve_file(&mut req, &server_config_list).await {
        Ok((status, headers, body)) => {
            let mut wr = req.response().status(status).await.headers(headers).await;
            if let Some(mut reader) = body {
                reader.read_to_writer(&mut wr).await;
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
