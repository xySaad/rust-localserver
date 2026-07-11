use std::{fs, io};

use rust_localserver::{
    connection_handler::connection_handler,
    future::{Pool, Task},
    http,
    parser::{self, ServerConfig},
};

async fn run_server(name: String, config: &ServerConfig) -> io::Result<()> {
    let server = http::Server::bind((config.address.as_ref(), config.port))?;
    println!(
        "[server.{name}] is listening at http://{}",
        server.listener.listener.local_addr()?
    );

    server.serve(&async |req| connection_handler(req, config).await).await;

    Ok(())
}

fn main() -> std::io::Result<()> {
    let config_str = fs::read_to_string("config.ini")?;
    let config = parser::parse_config(&config_str);

    match config {
        Ok(cfg) => {
            let mut servers_pool = Pool::new();
            for (name, server_config) in cfg.servers {
                let task = Task::new(async {
                    let name = name;
                    let server_config = server_config;
                    run_server(name.clone(), &server_config)
                        .await
                        .unwrap_or_else(|e| println!("[server.{name}]: {e}"))
                });
                servers_pool.add_task(task);
            }
            servers_pool.block();
        }
        Err(e) => println!("error parsing config: {e}"),
    }

    Ok(())
}
