use std::{collections::HashMap, format, fs, result};

use rust_localserver::{
    file_server::{FileServer, HTTPFileReader},
    future::{AsyncTcpStream, AsyncWrite, Pool, Task},
    http::{
        self, Headers, Request,
        Status::{self},
    },
    parser::{self, ServerConfig},
};

fn main() -> result::Result<(), String> {
    let config_str = fs::read_to_string("config.ini").map_err(|e| format!("error parsing config {e}"))?;
    let config = parser::parse_config(&config_str).map_err(|e| format!("error parsing config: {e}"))?;

    // Group server configs by their unique (address, port) combination
    //TODO: use hashmap instead of array to prevent two configs/servers with the  same host in the same binding
    //TODO: use resolved address instead of address string to handle address overlap (e.g, 0.0.0.0 with another address, or localhost with 127.0.0.1)
    let mut bindings = HashMap::<_, Vec<_>>::new();
    for (name, server_config) in config.servers {
        let key = (server_config.address.clone(), server_config.port);
        bindings.entry(key).or_default().push((name, server_config));
    }

    let mut servers_pool = Pool::new();
    for ((address, port), server_config_list) in bindings {
        //Host -> Server Name
        let mut seen_hosts: HashMap<String, String> = HashMap::new();

        for (name, cfg) in &server_config_list {
            for host in &cfg.host {
                if let Some(see_host_server_name) = seen_hosts.get(host) {
                    return Err(format!("duplicated host '{host}' in {name} and {see_host_server_name}"));
                };
                seen_hosts.insert(host.clone(), name.clone());
            }
        }

        let task = Task::new(async move {
            let handler = async |req: Request<&mut AsyncTcpStream>| match serve_file(&req, &server_config_list).await {
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

            match http::Server::bind((address.clone(), port)) {
                Err(e) => eprintln!("Error binding at {address}:{port} - {e}"),
                Ok(server) => {
                    if let Ok(addr) = server.listener.listener.local_addr() {
                        println!("Created binding at {}", addr);
                    } else {
                        println!("Created binding at {address}:{port}");
                    }
                    server_config_list.iter().for_each(|(name, cfg)| {
                        let label: Vec<_> = cfg.host.iter().map(|host| format!("http://{host}:{port}")).collect();
                        println!("[{name}]: {}", label.join(","))
                    });

                    server.serve(&handler).await;
                }
            };
        });
        servers_pool.add_task(task);
    }

    servers_pool.block();
    Ok(())
}

async fn serve_file<'t>(
    req: &Request<&mut AsyncTcpStream>,
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
        address: _,
        port: _,
        host: _,
    } = cfg;

    let file_server = FileServer::new(root, index, *list_directory);
    return file_server.serve(&req).await;
}
