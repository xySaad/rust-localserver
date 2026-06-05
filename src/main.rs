use std::fs;

pub mod future;
pub mod http;
pub mod parser;

fn main() -> std::io::Result<()> {
    let config_str = fs::read_to_string("config.ini")?;
    let config = parser::parse_config(&config_str);
    if let Ok(config) = config {
        for (name, server_config) in config.servers {
            http::Server::new(&name, (server_config.address, server_config.port))?;
        }
    } else {
        println!("{:?}", config.err())
    }
    Ok(())
}
