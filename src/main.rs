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
use std::sync::Arc;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    error::InternalError, http::StatusCode, middleware as actix_middleware,
    web::JsonConfig, App, HttpServer,
};
use lazy_static::lazy_static;
use log::info;

mod api;
mod archive;
mod data;
mod errors;
mod pages;
mod settings;
mod static_assets;
#[cfg(test)]
#[macro_use]
mod tests;

pub use crate::data::Data;
pub use api::v1::ROUTES as V1_API_ROUTES;
pub use pages::routes::PAGES;
pub use settings::Settings;
pub use static_assets::static_files::assets;

use static_assets::FileMap;

lazy_static! {
    pub static ref FILES: FileMap = FileMap::new();
    pub static ref CSS: &'static str =
        FILES.get("./static/cache/bundle/css/main.css").unwrap();
    pub static ref MOBILE_CSS: &'static str =
        FILES.get("./static/cache/bundle/css/mobile.css").unwrap();
    pub static ref JS: &'static str =
        FILES.get("./static/cache/bundle/bundle.js").unwrap();
    pub static ref GLUE: &'static str =
        FILES.get("./static/cache/bundle/glue.js").unwrap();
}

pub const CACHE_AGE: u32 = 604800;

pub const COMPILED_DATE: &str = env!("COMPILED_DATE");
pub const GIT_COMMIT_HASH: &str = env!("GIT_HASH");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
pub const PKG_DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
pub const PKG_HOMEPAGE: &str = env!("CARGO_PKG_HOMEPAGE");

pub type AppData = actix_web::web::Data<Arc<crate::data::Data>>;

#[cfg(not(tarpaulin_include))]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //env::set_var("RUST_LOG", "info");

    pretty_env_logger::init();

    info!(
        "{}: {}.\nFor more information, see: {}\nBuild info:\nVersion: {} commit: {}",
        PKG_NAME, PKG_DESCRIPTION, PKG_HOMEPAGE, VERSION, GIT_COMMIT_HASH
    );

    let settings = Settings::new().unwrap();
    let data = Data::new(settings.clone()).await;
    sqlx::migrate!("./migrations/").run(&data.db).await.unwrap();
    let data = actix_web::web::Data::new(data);

    let arch = archive::Archiver::new(&data.settings);
    arch.archive(&data).await.unwrap();

    let ip = settings.server.get_ip();
    println!("Starting server on: http://{}", ip);

    HttpServer::new(move || {
        App::new()
            .wrap(actix_middleware::Logger::default())
            .wrap(actix_middleware::Compress::default())
            .app_data(get_json_err())
            .wrap(
                actix_middleware::DefaultHeaders::new()
                    .add(("Permissions-Policy", "interest-cohort=()")),
            )
            .wrap(get_survey_session(&settings))
            .wrap(get_identity_service(&settings))
            .wrap(actix_middleware::NormalizePath::new(
                actix_middleware::TrailingSlash::Trim,
            ))
            .configure(services)
            .app_data(data.clone())
    })
    .bind(ip)
    .unwrap()
    .run()
    .await
}

#[cfg(not(tarpaulin_include))]
pub fn get_json_err() -> JsonConfig {
    JsonConfig::default().error_handler(|err, _| {
        //debug!("JSON deserialization error: {:?}", &err);
        InternalError::new(err, StatusCode::BAD_REQUEST).into()
    })
}

#[cfg(not(tarpaulin_include))]
pub fn get_survey_session(
    settings: &Settings,
) -> actix_session::SessionMiddleware<CookieSessionStore> {
    use actix_web::cookie::Key;
    let cookie_secret = &settings.server.cookie_secret2;
    let key = Key::from(cookie_secret.as_bytes());
    SessionMiddleware::builder(CookieSessionStore::default(), key)
        .cookie_domain(Some(settings.server.domain.clone()))
        .cookie_name("survey-id".into())
        .cookie_path("/".to_string())
        .cookie_secure(false)
        .cookie_http_only(true)
        .build()
}

#[cfg(not(tarpaulin_include))]
pub fn get_identity_service(
    settings: &Settings,
) -> IdentityService<CookieIdentityPolicy> {
    let cookie_secret = &settings.server.cookie_secret;
    IdentityService::new(
        CookieIdentityPolicy::new(cookie_secret.as_bytes())
            .path("/admin/")
            .name("survey-admin-auth")
            .max_age_secs(60 * 60 * 24 * 365)
            .domain(&settings.server.domain)
            .secure(false),
    )
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    api::v1::services(cfg);
    pages::services(cfg);
    static_assets::services(cfg);
}
