use std::fs;

pub mod parser;
pub mod server;

fn main() -> std::io::Result<()> {
    let config_str = fs::read_to_string("config.ini")?;
    let config = parser::parse_config(&config_str);
    if let Ok(config) = config {
        for (name, server_config) in config.servers {
            let address = format!("{}:{}", server_config.address, server_config.port);
            server::Server::new(&name, &address)?;
        }
    } else {
        println!("{:?}", config.err())
    }
    Ok(())
}
