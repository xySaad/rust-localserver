use std::{fs, io};

use crate::{
    future::{Pool, Task},
    parser::ServerConfig,
};

pub mod future;
pub mod http;
pub mod parser;

async fn run_server(name: String, config: ServerConfig) -> io::Result<()> {
    let server = http::Server::bind((config.address, config.port))?;
    println!(
        "[server.{name}] is listening at http://{}",
        server.listener.listener.local_addr()?
    );
    server.serve().await;

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
                    run_server(name.clone(), server_config)
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
