/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
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
use actix_auth_middleware::*;
use actix_web::web::ServiceConfig;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use serde::*;
use tera::*;

use crate::settings::Settings;
use crate::static_assets::routes::ASSETS;
use crate::{GIT_COMMIT_HASH, VERSION};

pub mod auth;
pub mod errors;
pub mod panel;
pub mod routes;

//pub use routes::get_auth_middleware;
pub use routes::PAGES;

pub struct TemplateFile {
    pub name: &'static str,
    pub path: &'static str,
}

impl TemplateFile {
    pub const fn new(name: &'static str, path: &'static str) -> Self {
        Self { name, path }
    }

    pub fn register(&self, t: &mut Tera) -> std::result::Result<(), tera::Error> {
        t.add_raw_template(self.name, &Templates::get_template(self).expect(self.name))
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub fn register_from_file(
        &self,
        t: &mut Tera,
    ) -> std::result::Result<(), tera::Error> {
        use std::path::Path;
        t.add_template_file(Path::new("templates/").join(self.path), Some(self.name))
    }
}

pub const PAYLOAD_KEY: &str = "payload";

pub const BASE: TemplateFile = TemplateFile::new("base", "components/base.html");
pub const FOOTER: TemplateFile =
    TemplateFile::new("footer", "components/footer/index.html");
pub const PANEL_NAV: TemplateFile =
    TemplateFile::new("panel_nav", "panel/nav/index.html");

lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        for t in [BASE, FOOTER, PANEL_NAV].iter() {
            t.register(&mut tera).unwrap();
        }
        errors::register_templates(&mut tera);
        tera.autoescape_on(vec![".html", ".sql"]);
        auth::register_templates(&mut tera);
        panel::register_templates(&mut tera);
        tera
    };
}

#[derive(RustEmbed)]
#[folder = "templates/"]
pub struct Templates;

impl Templates {
    pub fn get_template(t: &TemplateFile) -> Option<String> {
        match Self::get(t.path) {
            Some(file) => Some(String::from_utf8_lossy(&file.data).into_owned()),
            None => None,
        }
    }
}

pub fn context(s: &Settings, page_title: &str) -> Context {
    let mut ctx = Context::new();
    let footer = Footer::new(s);
    ctx.insert("page_title", page_title);
    ctx.insert("page", &PAGES);
    ctx.insert("api", &crate::V1_API_ROUTES);
    ctx.insert("footer", &footer);
    ctx.insert("assets", &*ASSETS);
    ctx
}

pub fn auth_ctx(_username: Option<&str>, s: &Settings, page_title: &str) -> Context {
    let mut ctx = Context::new();
    let footer = Footer::new(s);
    ctx.insert("page_title", page_title);
    ctx.insert("footer", &footer);
    ctx.insert("page", &PAGES);
    ctx.insert("api", &crate::V1_API_ROUTES);
    ctx.insert("assets", &*ASSETS);
    ctx
}

#[derive(Serialize)]
pub struct Footer<'a> {
    version: &'a str,
    support_email: &'a str,
    source_code: &'a str,
    git_hash: &'a str,
    settings: &'a Settings,
    about: &'a str,
    privacy: &'a str,
    donate: &'a str,
    security: &'a str,
    thanks: &'a str,
}

impl<'a> Footer<'a> {
    pub fn new(settings: &'a Settings) -> Self {
        Self {
            version: VERSION,
            source_code: &settings.source_code,
            support_email: &settings.support_email,
            git_hash: &GIT_COMMIT_HASH[..8],
            about: settings.footer.about.as_str(),
            privacy: settings.footer.privacy.as_str(),
            thanks: settings.footer.thanks.as_str(),
            security: settings.footer.security.as_str(),
            donate: settings.footer.donate.as_str(),
            settings,
        }
    }
}

pub fn services(cfg: &mut ServiceConfig) {
    auth::services(cfg);
    panel::services(cfg);
}

pub fn get_page_check_login() -> Authentication<auth::routes::Auth> {
    Authentication::with_identity(crate::PAGES.auth)
}

#[cfg(test)]
mod terra_tests {

    #[test]
    fn templates_work_basic() {
        use super::*;
        use tera::Tera;

        let mut tera = Tera::default();
        let mut tera2 = Tera::default();
        for t in [
            BASE,
            FOOTER,
            PANEL_NAV,
            auth::login::LOGIN,
            auth::join::REGISTER,
            errors::ERROR_TEMPLATE,
        ]
        .iter()
        {
            t.register_from_file(&mut tera2).unwrap();
            t.register(&mut tera).unwrap();
        }
    }
}

#[cfg(test)]
mod http_page_tests {
    use actix_web::http::{header, StatusCode};
    use actix_web::test;

    use crate::*;

    use super::PAGES;

    #[actix_rt::test]
    async fn templates_work() {
        use crate::tests::*;

        let data = get_test_data().await;
        let app = get_app!(data).await;

        for file in [PAGES.auth.login, PAGES.auth.join, PAGES.home].iter() {
            println!("[*] Testing route: {}", file);
            let resp = get_request!(&app, file);
            if file != &PAGES.home {
                assert_eq!(resp.status(), StatusCode::OK);
            } else {
                assert_eq!(resp.status(), StatusCode::FOUND);
                let headers = resp.headers();
                let loc = headers.get(header::LOCATION).unwrap();
                let resp = get_request!(&app, loc.to_str().unwrap());
                assert_eq!(resp.status(), StatusCode::OK);
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
#[cfg(test)]
mod tests {
    use actix_web::http::{header, StatusCode};
    use actix_web::test;

    use crate::tests::*;
    use crate::*;

    #[actix_rt::test]
    async fn protected_pages_templates_work() {
        const NAME: &str = "templateuser";
        const PASSWORD: &str = "longpassword";
        const EMAIL: &str = "templateuser@a.com";
        const CAMPAIGN_NAME: &str = "delcappageusercamaping";

        let data = get_test_data().await;
        {
            delete_user(NAME, &data).await;
        }

        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);

        let campaign =
            create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

        let app = get_app!(data).await;

        let urls = vec![
            PAGES.home.to_string(),
            PAGES.panel.campaigns.home.to_string(),
            PAGES.panel.campaigns.new.to_string(),
            //            PAGES.panel.campaigns.get_feedback_route(&campaign.uuid),
            PAGES
                .panel
                .campaigns
                .get_delete_route(&campaign.campaign_id),
        ];

        for url in urls.iter() {
            let resp =
                test::call_service(&app, test::TestRequest::get().uri(url).to_request())
                    .await;
            if resp.status() != StatusCode::FOUND {
                println!("Probably error url: {}", url);
            }
            assert_eq!(resp.status(), StatusCode::FOUND);

            let authenticated_resp = test::call_service(
                &app,
                test::TestRequest::get()
                    .uri(url)
                    .cookie(cookies.clone())
                    .to_request(),
            )
            .await;

            if url == PAGES.home {
                assert_eq!(authenticated_resp.status(), StatusCode::FOUND);
                let headers = authenticated_resp.headers();
                assert_eq!(
                    headers.get(header::LOCATION).unwrap().to_str().unwrap(),
                    PAGES
                        .panel
                        .campaigns
                        .get_about_route(&data.settings.default_campaign)
                );
            } else {
                assert_eq!(authenticated_resp.status(), StatusCode::OK);
            }
        }

        delete_user(NAME, &data).await;
    }

    #[actix_rt::test]
    async fn public_pages_tempaltes_work() {
        let data = get_test_data().await;
        let app = get_app!(data).await;
        let urls = vec![PAGES.auth.login, PAGES.auth.join]; //, PAGES.sitemap];
        for url in urls.iter() {
            let resp =
                test::call_service(&app, test::TestRequest::get().uri(url).to_request())
                    .await;

            assert_eq!(resp.status(), StatusCode::OK);
        }
    }
}
