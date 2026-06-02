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

fn get_value(options: &HashMap<String, String>, option_name: &str) -> Result<String, String> {
    let value = options
        .get(option_name)
        .ok_or(format!("missing required option: {}", option_name))?
        .clone();

    return Ok(value);
}

fn parse_server_config(options: &HashMap<String, String>) -> Result<ServerConfig, String> {
    let address = get_value(options, "address")?;
    let host = get_value(options, "host")?;
    let root_destination = get_value(options, "root_destination")?;
    let port = get_value(options, "port")?.parse::<u16>().map_err(|x| x.to_string())?;

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

    for (section, options_list) in ini_config.sections {
        let parts: Vec<_> = section.split(".").collect();

        if is_server(&parts) {
            if parts.len() == 2 {
                //global server config
                if options_list.len() > 1 {
                    return Err(format!("Duplicated [server.{}] config", parts[1]));
                }
                if options_list.len() < 1 {
                    return Err(format!("Missing [server.{}] config", parts[1]));
                }
                let server_name = parts[1].to_owned();
                let server_config = parse_server_config(&options_list[0])?;
                servers.insert(server_name, server_config);
            }
        }

        if is_client(&parts) {
            if options_list.len() > 1 {
                return Err(format!("Duplicated [client] config"));
            }
            if options_list.len() < 1 {
                return Err(format!("Missing [client] config"));
            }
            let body_size_limit = get_value(&options_list[0], "body_size_limit")?
                .parse::<u64>()
                .map_err(|x| x.to_string())?;
            client_config = Some(ClientConfig { body_size_limit })
        }
    }

    if let Some(client) = client_config {
        return Ok(MuxConfig { client, servers });
    }

    return Err(format!("Missing [client] config"));
}
