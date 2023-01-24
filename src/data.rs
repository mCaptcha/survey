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
//! App data: database connections, etc.
use std::sync::Arc;
use std::thread;

use argon2_creds::{Config, ConfigBuilder, PasswordPolicy};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;

use crate::settings::Settings;

/// App data
pub struct Data {
    /// database pool
    pub db: PgPool,
    pub creds: Config,
    pub settings: Settings,
}

impl Data {
    pub fn get_creds() -> Config {
        ConfigBuilder::default()
            .username_case_mapped(true)
            .profanity(true)
            .blacklist(true)
            .password_policy(PasswordPolicy::default())
            .build()
            .unwrap()
    }

    #[cfg(not(tarpaulin_include))]
    /// create new instance of app data
    pub async fn new(settings: Settings) -> Arc<Self> {
        let creds = Self::get_creds();
        let c = creds.clone();
        #[allow(unused_variables)]
        let init = thread::spawn(move || {
            log::info!("Initializing credential manager");
            c.init();
            log::info!("Initialized credential manager");
        });

        let db = PgPoolOptions::new()
            .max_connections(settings.database.pool)
            .connect(&settings.database.url)
            .await
            .expect("Unable to form database pool");

        #[cfg(not(debug_assertions))]
        init.join().unwrap();
        let data = Data {
            db,
            creds,
            settings,
        };

        Arc::new(data)
    }
}
