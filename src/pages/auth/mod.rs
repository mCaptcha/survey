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
pub mod join;
pub mod login;
pub mod sudo;

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(login::login);
    cfg.service(login::login_submit);
    cfg.service(join::join);
    cfg.service(join::join_submit);
}

pub mod routes {
    use crate::middleware::auth::GetLoginRoute;
    use url::Url;

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
                login: "/api/v1/admin/page/login",
                join: "/api/v1/admin/page/join",
            }
        }

        pub const fn get_sitemap() -> [&'static str; 2] {
            const AUTH: Auth = Auth::new();
            [AUTH.login, AUTH.join]
        }
    }
}
