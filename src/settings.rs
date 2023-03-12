/*
 * Copyright (C) 2021  Aravinth Manivannan <realaravinth@batsense.net>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as
 * published by the Free Software Foundation, either version 3 of the
 * License, or (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::env;
use std::fs;
use std::path::Path;

use config::{Config, ConfigError, Environment, File};
use log::{debug, warn};
use serde::Deserialize;
use serde::Serialize;
use sqlx::types::Uuid;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Server {
    pub port: u32,
    pub domain: String,
    pub cookie_secret: String,
    pub cookie_secret2: String,
    pub ip: String,
    pub proxy_has_tls: bool,
}

impl Server {
    #[cfg(not(tarpaulin_include))]
    pub fn get_ip(&self) -> String {
        format!("{}:{}", self.ip, self.port)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseBuilder {
    pub port: u32,
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub name: String,
    pub url: String,
}

impl DatabaseBuilder {
    #[cfg(not(tarpaulin_include))]
    fn extract_database_url(url: &Url) -> Self {
        debug!("Databse name: {}", url.path());
        let mut path = url.path().split('/');
        path.next();
        let name = path.next().expect("no database name").to_string();
        DatabaseBuilder {
            port: url.port().expect("Enter database port").into(),
            hostname: url.host().expect("Enter database host").to_string(),
            username: url.username().into(),
            url: url.to_string(),
            password: url.password().expect("Enter database password").into(),
            name,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footer {
    pub about: Url,
    pub privacy: Url,
    pub security: Url,
    pub donate: Url,
    pub thanks: Url,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Publish {
    pub dir: String,
    pub duration: u64,
}

impl Publish {
    fn create_root_dir(&self) {
        let root = Path::new(&self.dir);
        if root.exists() {
            if !root.is_dir() {
                std::fs::remove_file(&root).unwrap();
                std::fs::create_dir_all(&root).unwrap();
            }
        } else {
            std::fs::create_dir_all(&root).unwrap();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub debug: bool,
    pub allow_registration: bool,
    pub database: Database,
    pub server: Server,
    pub source_code: String,
    pub support_email: String,
    pub default_campaign: String,
    pub footer: Footer,
    pub publish: Publish,
}

#[cfg(not(tarpaulin_include))]
impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::new();

        // setting default values
        #[cfg(test)]
        s.set_default("database.pool", 2.to_string())
            .expect("Couldn't get the number of CPUs");

        const CURRENT_DIR: &str = "./config/default.toml";
        const ETC: &str = "/etc/mcaptcha-survey/config.toml";

        if let Ok(path) = env::var("ATHENA_CONFIG") {
            s.merge(File::with_name(&path))?;
        } else if Path::new(CURRENT_DIR).exists() {
            // merging default config from file
            s.merge(File::with_name(CURRENT_DIR))?;
        } else if Path::new(ETC).exists() {
            s.merge(File::with_name(ETC))?;
        } else {
            log::warn!("configuration file not found");
        }

        s.merge(Environment::with_prefix("MCAPTCHA").separator("__"))?;

        check_url(&s);
        check_uuid(&s);

        match env::var("PORT") {
            Ok(val) => {
                s.set("server.port", val).unwrap();
            }
            Err(e) => warn!("couldn't interpret PORT: {}", e),
        }

        match env::var("DATABASE_URL") {
            Ok(val) => {
                let url = Url::parse(&val).expect("couldn't parse Database URL");
                let database_conf = DatabaseBuilder::extract_database_url(&url);
                set_from_database_url(&mut s, &database_conf);
            }
            Err(e) => warn!("couldn't interpret DATABASE_URL: {}", e),
        }

        set_database_url(&mut s);

        match s.try_into::<Settings>() {
            Ok(val) => {
                val.publish.create_root_dir();
                Ok(val)
            },
            Err(e) => Err(ConfigError::Message(format!("\n\nError: {}. If it says missing fields, then please refer to https://github.com/mCaptcha/mcaptcha#configuration to learn more about how mcaptcha reads configuration\n\n", e))),
        }
    }
}

#[cfg(not(tarpaulin_include))]
fn check_url(s: &Config) {
    let url = s
        .get::<String>("source_code")
        .expect("Couldn't access source_code");

    Url::parse(&url).expect("Please enter a URL for source_code in settings");
}

#[cfg(not(tarpaulin_include))]
fn check_uuid(s: &Config) {
    use std::str::FromStr;

    let id = s
        .get::<String>("default_campaign")
        .expect("Couldn't access default_campaign");

    Uuid::from_str(&id).expect("Please enter a UUID for default_campaign in settings");
}

#[cfg(not(tarpaulin_include))]
fn set_from_database_url(s: &mut Config, database_conf: &DatabaseBuilder) {
    s.set("database.username", database_conf.username.clone())
        .expect("Couldn't set database username");
    s.set("database.password", database_conf.password.clone())
        .expect("Couldn't access database password");
    s.set("database.hostname", database_conf.hostname.clone())
        .expect("Couldn't access database hostname");
    s.set("database.port", database_conf.port as i64)
        .expect("Couldn't access database port");
    s.set("database.name", database_conf.name.clone())
        .expect("Couldn't access database name");
}

#[cfg(not(tarpaulin_include))]
fn set_database_url(s: &mut Config) {
    s.set(
        "database.url",
        format!(
            r"postgres://{}:{}@{}:{}/{}",
            s.get::<String>("database.username")
                .expect("Couldn't access database username"),
            s.get::<String>("database.password")
                .expect("Couldn't access database password"),
            s.get::<String>("database.hostname")
                .expect("Couldn't access database hostname"),
            s.get::<String>("database.port")
                .expect("Couldn't access database port"),
            s.get::<String>("database.name")
                .expect("Couldn't access database name")
        ),
    )
    .expect("Couldn't set databse url");
}
