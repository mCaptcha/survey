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
 * You should have received a copy of the GNU Affero General Public License along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */
use std::cell::RefCell;

use actix_web::http::header::ContentType;
use actix_web::{HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tera::Context;

use crate::api::v1::admin::campaigns::{runners, ListCampaignResp};
use crate::settings::Settings;
use crate::AppData;

use super::*;
pub use crate::pages::errors::*;

pub struct ExportPage {
    ctx: RefCell<Context>,
}

pub const EXPORT_CAMPAIGNS: TemplateFile =
    TemplateFile::new("export_campaigns", "panel/export.html");

impl CtxError for ExportPage {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ExportPagePayload {
    pub campaigns: Vec<ExportListCampaign>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ExportListCampaign {
    pub uuid: String,
    pub name: String,
    pub export_link: String,
    pub route: String,
}

impl From<ListCampaignResp> for ExportListCampaign {
    fn from(value: ListCampaignResp) -> Self {
        use crate::DOWNLOAD_SCOPE;

        let route = PAGES.panel.campaigns.get_bench_route(&value.uuid);
        let export_link = format!("{DOWNLOAD_SCOPE}/{}", value.uuid);
        Self {
            uuid: value.uuid,
            name: value.name,
            export_link,
            route,
        }
    }
}

impl ExportPage {
    pub fn new(settings: &Settings, payload: Option<ExportPagePayload>) -> Self {
        let ctx = RefCell::new(context(settings, "Results"));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(EXPORT_CAMPAIGNS.name, &self.ctx.borrow())
            .unwrap()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.panel.export")]
pub async fn export_campaigns(data: AppData) -> PageResult<impl Responder, ExportPage> {
    let mut campaigns = runners::list_all_campaigns(&data)
        .await
        .map_err(|e| PageError::new(ExportPage::new(&data.settings, None), e))?;
    let campaigns = campaigns.drain(0..).map(|c| c.into()).collect();
    let payload = ExportPagePayload { campaigns };

    let results_page = ExportPage::new(&data.settings, Some(payload)).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(results_page))
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(export_campaigns);
}

#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;

    use crate::tests::*;
    use crate::*;
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn export_campaigns_works() {
        const NAME: &str = "pubexportcampaignuser";
        const EMAIL: &str = "pubexportcampaignuser@aaa.com";
        const PASSWORD: &str = "longpassword";
        const CAMPAIGN_NAME: &str = "pubexportcampaignusercampaign";

        let data = get_test_data().await;
        let app = get_app!(data).await;
        delete_user(NAME, &data).await;
        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);

        let uuid =
            create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

        let home_resp = get_request!(
            &app,
            &PAGES
                .panel
                .campaigns
                .get_about_route(&data.settings.default_campaign)
        );

        assert_eq!(home_resp.status(), StatusCode::OK);
        let body = String::from_utf8(test::read_body(home_resp).await.to_vec()).unwrap();
        assert!(body.contains(PAGES.panel.export));

        let export_page_resp = get_request!(&app, PAGES.panel.export);
        assert_eq!(export_page_resp.status(), StatusCode::OK);
        let body =
            String::from_utf8(test::read_body(export_page_resp).await.to_vec()).unwrap();
        assert!(body.contains(&uuid.campaign_id.to_string()));
    }
}
