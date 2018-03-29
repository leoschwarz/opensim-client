//! Read client configuration from a file.
//!
//! Note: This is mostly used to make debugging as painless as possible,
//!       and might not really represent what we want to have in the final
//!       viewer at all.

use std::fs::File;
use std::io::Read;
use std::path::Path;
use toml;

#[derive(Deserialize)]
pub struct Config {
    pub user: ConfigUser,
    pub sim: ConfigSim,
}

#[derive(Deserialize)]
pub struct ConfigUser {
    pub first_name: String,
    pub last_name: String,
    pub password_plain: String,
}

#[derive(Deserialize)]
pub struct ConfigSim {
    pub loginuri: String,
}

pub fn get_config<P: AsRef<Path>>(path: P) -> Result<Config, String> {
    let mut file = File::open(path.as_ref())
        .map_err(|_| format!("Failed reading config file {:?}", path.as_ref()))?;
    let mut raw_data = String::new();
    file.read_to_string(&mut raw_data).unwrap();
    toml::from_str(raw_data.as_str()).map_err(|e| format!("Invalid TOML: {}", e))
}
