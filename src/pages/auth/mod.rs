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

pub use crate::api::v1::admin::get_admin_check_login;

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(login::login);
    cfg.service(login::login_submit);
    cfg.service(join::join);
    cfg.service(join::join_submit);
}

pub mod routes {
    pub struct Auth {
        pub login: &'static str,
        pub join: &'static str,
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
