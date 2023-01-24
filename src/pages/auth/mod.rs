/*
 * Copyright (C) 2023  Aravinth Manivannan <realaravinth@batsense.net>
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
use actix_web::*;

pub use super::{context, Footer, TemplateFile, PAGES, PAYLOAD_KEY, TEMPLATES};

pub fn register_templates(t: &mut tera::Tera) {
    for template in [login::LOGIN, join::REGISTER].iter() {
        template.register(t).expect(template.name);
    }
}

pub mod join;
pub mod login;
//pub mod sudo;

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    login::services(cfg);
    join::services(cfg);
}

pub mod routes {
    use actix_auth_middleware::GetLoginRoute;
    use serde::{Deserialize, Serialize};
    use url::Url;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub struct Auth {
        pub login: &'static str,
        pub join: &'static str,
    }

    impl GetLoginRoute for Auth {
        fn get_login_route(&self, src: Option<&str>) -> String {
            if let Some(redirect_to) = src {
                let mut url = Url::parse("http://x/").unwrap();
                url.set_path(self.login);
                url.query_pairs_mut()
                    .append_pair("redirect_to", redirect_to);
                let path = format!("{}/?{}", url.path(), url.query().unwrap());
                path
            } else {
                self.login.to_string()
            }
        }
    }

    impl Auth {
        pub const fn new() -> Auth {
            Auth {
                login: "/admin/login",
                join: "/admin/join",
            }
        }

        pub const fn get_sitemap() -> [&'static str; 2] {
            const AUTH: Auth = Auth::new();
            [AUTH.login, AUTH.join]
        }
    }
}
