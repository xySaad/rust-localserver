use crate::{
    file_server::FileServer,
    future::AsyncWrite,
    http::{self, Headers, Request, Status},
    parser::ServerConfig,
};
//gha 9di wsf hadxi li 3ta lah
fn access_control(req: &Request<'_>, config: &ServerConfig) -> http::Result {
    let ServerConfig { host, .. } = config;
    let req_host = req.header("Host").map(|s| s.to_owned()).ok_or(Status::BadRequest)?;
    if !host.contains(&req_host) {
        return Err(Status::MisdirectedRequest);
    }
    Ok(())
}

pub async fn connection_handler(req: Request<'_>, config: &ServerConfig) {
    if let Err(status) = access_control(&req, config) {
        let body = format!("{0} {0:?}", status);
        _ = req
            .response()
            .status(status)
            .await
            .headers(Headers::new())
            .await
            .write(body.as_bytes())
            .await;
        return;
    }
    let ServerConfig {
        root,
        list_directory,
        index,
        ..
    } = config;

    let file_server = FileServer::new(root, index, *list_directory);
    file_server.serve(req).await;
}
