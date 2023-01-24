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
use actix_web::{http, HttpResponse, Responder};

pub use super::{context, Footer, TemplateFile, PAGES, PAYLOAD_KEY, TEMPLATES};
use crate::AppData;

mod campaigns;

pub fn register_templates(t: &mut tera::Tera) {
    campaigns::register_templates(t);
    //    for template in [REGISTER].iter() {
    //        template.register(t).expect(template.name);
    //    }
}

pub mod routes {
    use super::campaigns::routes::Campaigns;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
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

#[actix_web_codegen_const_routes::get(path = "PAGES.panel.home")]
pub async fn home(ctx: AppData) -> impl Responder {
    let loc = PAGES
        .panel
        .campaigns
        .get_about_route(&ctx.settings.default_campaign);

    HttpResponse::Found()
        .insert_header((http::header::LOCATION, loc))
        .finish()
}
