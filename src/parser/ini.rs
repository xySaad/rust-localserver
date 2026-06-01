use std::{
    collections::{BTreeMap, HashMap},
    str::Split,
};

pub type INIOptions = HashMap<String, String>;
pub type INISection = Vec<INIOptions>;

#[derive(Debug)]
pub struct INIConfig {
    pub global_options: INIOptions,
    pub sections: BTreeMap<String, INISection>,
}

pub fn parse_ini(config_data: &str) -> INIConfig {
    let mut ini_config = INIConfig {
        global_options: INIOptions::new(),
        sections: BTreeMap::new(),
    };

    let mut lines = config_data.split("\n");
    let (mut options, mut next_section) = parse_options(&mut lines);
    ini_config.global_options = options;

    loop {
        if let Some(section) = next_section {
            (options, next_section) = parse_options(&mut lines);
            ini_config
                .sections
                .entry(section.to_owned())
                .or_insert(Vec::new())
                .push(options);
        } else {
            break;
        }
    }
    return ini_config;
}

pub fn parse_option(line: &str) -> (Option<&str>, Option<&str>) {
    let mut parts = line.split('=');
    let key_opt = parts.next();
    let value_opt = parts.next();
    return (key_opt, value_opt);
}

pub fn parse_options<'t>(lines: &mut Split<'t, &str>) -> (INIOptions, Option<&'t str>) {
    let mut options = INIOptions::new();

    for mut line in lines {
        line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            let section = &line[1..line.len() - 1];
            return (options, Some(section));
        }

        if let (Some(key), Some(value)) = parse_option(line) {
            options.insert(key.to_string(), value.to_string());
        }
    }
    return (options, None);
}
