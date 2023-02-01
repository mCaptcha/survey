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
use std::cell::RefCell;

use actix_identity::Identity;
use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::api::v1::admin::campaigns::ResultsPage;
use crate::api::v1::admin::campaigns::{
    runners::list_campaign_runner, ListCampaignResp,
};
use crate::api::v1::bench::SubmissionType;
use crate::pages::errors::*;
use crate::AppData;
use crate::Settings;

pub mod about;
pub mod bench;
pub mod delete;
pub mod new;
pub mod results;

pub use super::{context, Footer, TemplateFile, PAGES, PAYLOAD_KEY, TEMPLATES};

pub fn register_templates(t: &mut tera::Tera) {
    for template in [
        CAMPAIGNS,
        about::INTRO,
        new::NEW_CAMPAIGN,
        new::NEW_CAMPAIGN_FORM,
        bench::BENCH,
        delete::SUDO_DELETE,
        results::CAMPAIGN_RESULTS,
    ]
    .iter()
    {
        template.register(t).expect(template.name);
    }
}

pub mod routes {
    use serde::{Deserialize, Serialize};

    use crate::api::v1::admin::campaigns::ResultsPage;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub struct Campaigns {
        pub home: &'static str,
        pub new: &'static str,
        pub about: &'static str,
        pub bench: &'static str,
        pub delete: &'static str,
        pub results: &'static str,
    }

    impl Campaigns {
        pub const fn new() -> Campaigns {
            Campaigns {
                home: "/admin/campaigns",
                new: "/admin/campaigns/new",
                about: "/survey/campaigns/{uuid}/about",
                bench: "/survey/campaigns/{uuid}/bench",
                delete: "/admin/campaigns/{uuid}/delete",
                results: "/admin/campaigns/{uuid}/results",
            }
        }

        pub fn get_delete_route(&self, campaign_id: &str) -> String {
            self.delete.replace("{uuid}", campaign_id)
        }

        pub fn get_bench_route(&self, campaign_id: &str) -> String {
            self.bench.replace("{uuid}", campaign_id)
        }

        pub fn get_about_route(&self, campaign_id: &str) -> String {
            self.about.replace("{uuid}", campaign_id)
        }

        pub fn get_results_route(
            &self,
            campaign_id: &str,
            modifier: Option<ResultsPage>,
        ) -> String {
            let mut res = self.results.replace("{uuid}", campaign_id);
            if let Some(modifier) = modifier {
                let page = modifier.page();
                if page != 0 {
                    res = format!("{res}?page={page}");
                }

                if let Some(bench_type) = modifier.bench_type {
                    if page != 0 {
                        res = format!("{res}&bench_type={}", bench_type.to_string());
                    } else {
                        res = format!("{res}?bench_type={}", bench_type.to_string());
                    }
                }
            }
            res
        }

        pub const fn get_sitemap() -> [&'static str; 2] {
            const CAMPAIGNS: Campaigns = Campaigns::new();
            [CAMPAIGNS.home, CAMPAIGNS.new]
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(home);
    about::services(cfg);
    new::services(cfg);
    bench::services(cfg);
    delete::services(cfg);
    results::services(cfg);
}

pub use super::*;

pub struct Campaigns {
    ctx: RefCell<Context>,
}

pub const CAMPAIGNS: TemplateFile =
    TemplateFile::new("campaigns", "panel/campaigns/index.html");

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct TemplateCampaign {
    pub name: String,
    pub uuid: String,
    pub route: String,
    pub results: String,
}

impl From<ListCampaignResp> for TemplateCampaign {
    fn from(c: ListCampaignResp) -> Self {
        let route = crate::PAGES.panel.campaigns.get_about_route(&c.uuid);
        let results = crate::PAGES
            .panel
            .campaigns
            .get_results_route(&c.uuid, None);
        let uuid = c.uuid;
        let name = c.name;
        Self {
            route,
            name,
            uuid,
            results,
        }
    }
}

impl CtxError for Campaigns {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Campaigns {
    pub fn new(settings: &Settings, payload: Option<Vec<TemplateCampaign>>) -> Self {
        let ctx = RefCell::new(context(settings, "Campaigns"));
        if let Some(payload) = payload {
            if !payload.is_empty() {
                ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
            }
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(CAMPAIGNS.name, &self.ctx.borrow())
            .unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s, None);
        p.render()
    }
}

#[actix_web_codegen_const_routes::get(
    path = "PAGES.panel.campaigns.home",
    wrap = "crate::pages::get_page_check_login()"
)]
pub async fn home(data: AppData, id: Identity) -> PageResult<impl Responder, Campaigns> {
    let username = id.identity().unwrap();
    let mut campaigns = list_campaign_runner(&username, &data)
        .await
        .map_err(|e| PageError::new(Campaigns::new(&data.settings, None), e))?;
    let mut template_campaigns = Vec::with_capacity(campaigns.len());
    for c in campaigns.drain(0..) {
        template_campaigns.push(c.into())
    }

    let list_campaigns =
        Campaigns::new(&data.settings, Some(template_campaigns)).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(list_campaigns))
}

#[cfg(test)]
mod tests {
    use actix_web::cookie::Cookie;
    use actix_web::http::StatusCode;
    use actix_web::test;

    use crate::tests::*;
    use crate::*;

    async fn protect_urls_test(urls: &[String], data: Arc<Data>, cookie: Cookie<'_>) {
        let app = get_app!(data).await;
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
                    .cookie(cookie.clone())
                    .to_request(),
            )
            .await;

            assert_eq!(authenticated_resp.status(), StatusCode::OK);
        }
    }

    #[actix_rt::test]
    async fn survey_pages_work() {
        const NAME: &str = "surveyuserpages";
        const PASSWORD: &str = "longpassword";
        const EMAIL: &str = "templateuser@surveyuserpages.com";
        const CAMPAIGN_NAME: &str = "delcappageusercamaping";

        let data = get_test_data().await;
        {
            delete_user(NAME, &data).await;
        }

        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let survey = get_survey_user(data.clone()).await;
        let survey_cookie = get_cookie!(survey);

        let campaign =
            create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

        let app = get_app!(data).await;

        let survey_protected_urls =
            vec![PAGES.panel.campaigns.get_bench_route(&campaign.campaign_id)];

        let public_urls =
            vec![PAGES.panel.campaigns.get_about_route(&campaign.campaign_id)];

        for url in public_urls.iter() {
            let resp =
                test::call_service(&app, test::TestRequest::get().uri(url).to_request())
                    .await;
            if resp.status() != StatusCode::OK {
                println!("Probably error url: {}", url);
            }
            assert_eq!(resp.status(), StatusCode::OK);
        }

        protect_urls_test(&survey_protected_urls, data, survey_cookie).await;
    }
}
