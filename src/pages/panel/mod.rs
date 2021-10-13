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
 * You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use actix_web::{HttpResponse, Responder};
use lazy_static::lazy_static;
use my_codegen::get;
use sailfish::TemplateOnce;

use crate::PAGES;

mod campaigns;

pub mod routes {
    use super::campaigns::routes::Campaigns;

    pub struct Panel {
        pub home: &'static str,
        pub campaigns: Campaigns,
    }
    impl Panel {
        pub const fn new() -> Panel {
            let campaigns = Campaigns::new();
            Panel {
                home: "/",
                campaigns,
            }
        }

        pub const fn get_sitemap() -> [&'static str; 1] {
            const PANEL: Panel = Panel::new();
            [PANEL.home]
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(home);
    campaigns::services(cfg);
}

#[derive(TemplateOnce, Default)]
#[template(path = "index.html")]
struct HomePage;

const PAGE: &str = "Survey";

lazy_static! {
    static ref INDEX: String = HomePage::default().render_once().unwrap();
}

#[get(path = "PAGES.panel.home")]
pub async fn home() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&*INDEX)
}
