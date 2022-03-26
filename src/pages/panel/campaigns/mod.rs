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

use crate::api::v1::admin::campaigns::{
    runners::list_campaign_runner, ListCampaignResp,
};
use crate::AppData;
use crate::PAGES;

pub mod about;
pub mod bench;
pub mod delete;
pub mod new;

pub mod routes {
    pub struct Campaigns {
        pub home: &'static str,
        pub new: &'static str,
        pub about: &'static str,
        pub bench: &'static str,
        pub delete: &'static str,
    }
    impl Campaigns {
        pub const fn new() -> Campaigns {
            Campaigns {
                home: "/admin/campaigns",
                new: "/admin/campaigns/new",
                about: "/survey/campaigns/{uuid}/about",
                bench: "/survey/campaigns/{uuid}/bench",
                delete: "/admin/campaigns/{uuid}/delete",
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
    cfg.service(about::about);
    cfg.service(bench::bench);
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

#[get(
    path = "PAGES.panel.campaigns.home",
    wrap = "crate::pages::get_page_check_login()"
)]
pub async fn home(data: AppData, id: Identity) -> impl Responder {
    let username = id.identity().unwrap();
    let campaigns = list_campaign_runner(&username, &data).await.unwrap();
    let page = HomePage::new(campaigns).render_once().unwrap();

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
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

        let data = Data::new().await;
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
