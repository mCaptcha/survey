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
use actix_web::web::ServiceConfig;

pub mod account;
pub mod auth;
pub mod campaigns;
#[cfg(test)]
mod tests;

pub use super::{get_random, get_uuid};

pub fn services(cfg: &mut ServiceConfig) {
    auth::services(cfg);
    account::services(cfg);
}

pub fn get_admin_check_login() -> crate::CheckLogin {
    crate::CheckLogin::new(crate::V1_API_ROUTES.admin.auth.register)
}

pub mod routes {
    use super::account::routes::Account;
    use super::auth::routes::Auth;
    use super::campaigns::routes::Campaign;

    pub struct Admin {
        pub auth: Auth,
        pub account: Account,
        pub campaign: Campaign,
    }

    impl Admin {
        pub const fn new() -> Admin {
            Admin {
                account: Account::new(),
                auth: Auth::new(),
                campaign: Campaign::new(),
            }
        }
    }
}
