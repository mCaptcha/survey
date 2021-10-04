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
use actix_identity::Identity;
use actix_web::{HttpResponse, Responder};
use my_codegen::get;
use sailfish::TemplateOnce;

use crate::api::v1::campaign::{runners::list_campaign_runner, ListCampaignResp};
use crate::AppData;
use crate::PAGES;

pub mod delete;
pub mod get;
pub mod new;

pub mod routes {
    pub struct Campaigns {
        pub home: &'static str,
        pub new: &'static str,
        pub get_feedback: &'static str,
        pub delete: &'static str,
    }
    impl Campaigns {
        pub const fn new() -> Campaigns {
            Campaigns {
                home: "/campaigns",
                new: "/campaigns/new",
                get_feedback: "/campaigns/{uuid}/feedback",
                delete: "/campaigns/{uuid}/delete",
            }
        }

        pub fn get_delete_route(&self, campaign_id: &str) -> String {
            self.delete.replace("{uuid}", campaign_id)
        }

        pub fn get_feedback_route(&self, campaign_id: &str) -> String {
            self.get_feedback.replace("{uuid}", campaign_id)
        }

        pub const fn get_sitemap() -> [&'static str; 2] {
            const CAMPAIGNS: Campaigns = Campaigns::new();
            [CAMPAIGNS.home, CAMPAIGNS.new]
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(home);
    cfg.service(new::new_campaign);
    cfg.service(new::new_campaign_submit);
    cfg.service(get::get_feedback);
    cfg.service(delete::delete_campaign);
    cfg.service(delete::delete_campaign_submit);
}

#[derive(TemplateOnce)]
#[template(path = "panel/campaigns/index.html")]
struct HomePage {
    data: Vec<ListCampaignResp>,
}

impl HomePage {
    fn new(data: Vec<ListCampaignResp>) -> Self {
        Self { data }
    }
}

const PAGE: &str = "Campaigns";

#[get(path = "PAGES.panel.campaigns.home", wrap = "crate::CheckLogin")]
pub async fn home(data: AppData, id: Identity) -> impl Responder {
    let username = id.identity().unwrap();
    let campaigns = list_campaign_runner(&username, &data).await.unwrap();
    let page = HomePage::new(campaigns).render_once().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&page)
}
