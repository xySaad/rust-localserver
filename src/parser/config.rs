use std::collections::HashMap;

use crate::parser::parse_ini;

pub struct ClientConfig {
    pub body_size_limit: u64,
}

pub struct ServerConfig {
    pub root_destination: String,
    pub address: String,
    pub port: u16,
    pub host: String,
}

pub struct MuxConfig {
    pub client: ClientConfig,
    pub servers: HashMap<String, ServerConfig>,
}

fn match_at(parts: &Vec<&str>, i: usize, str: &str) -> bool {
    parts.get(i).is_some_and(|v| *v == str)
}

fn is_server(parts: &Vec<&str>) -> bool {
    match_at(parts, 0, "server")
}
fn is_client(parts: &Vec<&str>) -> bool {
    match_at(parts, 0, "client")
}

fn parse_server_config(value: &HashMap<String, String>) -> Result<ServerConfig, String> {
    let address = value
        .get("address")
        .ok_or("missing required field: address".to_string())?
        .clone();

    let host = value
        .get("host")
        .ok_or("missing required field: host".to_string())?
        .clone();

    let root_destination = value
        .get("host")
        .ok_or("missing required field: root_destination".to_string())?
        .clone();

    let port = value
        .get("port")
        .ok_or("missing required field: port".to_string())?
        .parse::<u16>()
        .map_err(|_| "invalid port".to_string())?;

    let server_config = ServerConfig {
        address,
        host,
        port,
        root_destination,
    };

    return Ok(server_config);
}

pub fn parse_config(config_data: &str) -> Result<MuxConfig, String> {
    let ini_config = parse_ini(config_data);
    let mut client_config: Option<ClientConfig> = None;
    let mut servers = HashMap::<String, ServerConfig>::new();

    for (section, value) in ini_config.sections {
        let parts: Vec<_> = section.split(".").collect();

        if is_server(&parts) {
            if parts.len() == 2 {
                //global server config
                if value.len() > 1 {
                    return Err(format!("Duplicated [server.{}] config", parts[1]));
                }
                if value.len() < 1 {
                    return Err(format!("Missing [server.{}] config", parts[1]));
                }
                let server_name = parts[1].to_owned();
                let server_config = parse_server_config(&value[0])?;
                servers.insert(server_name, server_config);
            }
        }

        if is_client(&parts) {
            if value.len() > 1 {
                return Err(format!("Duplicated [client] config"));
            }
            if value.len() < 1 {
                return Err(format!("Missing [client] config"));
            }
            let body_size_limit = value[0]
                .get("body_size_limit")
                .ok_or("missing required field: body_size_limit".to_string())?
                .parse::<u64>()
                .map_err(|_| "invalid port".to_string())?;

            client_config = Some(ClientConfig { body_size_limit })
        }
    }

    if let Some(client) = client_config {
        return Ok(MuxConfig { client, servers });
    }

    return Err(format!("Missing [client] config"));
}
