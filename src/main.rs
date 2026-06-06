use std::fs;

pub mod future;
pub mod http;
pub mod parser;

fn main() -> std::io::Result<()> {
    let config_str = fs::read_to_string("config.ini")?;
    let config = parser::parse_config(&config_str);
    if let Ok(config) = config {
        for (name, server_config) in config.servers {
            let server = http::Server::bind((server_config.address, server_config.port))?;
            println!(
                "[server.{name}] is listening at http://{}",
                server.listener.listener.local_addr()?
            );
            server.serve_and_block();
        }
    } else {
        println!("{:?}", config.err())
    }
    Ok(())
}
