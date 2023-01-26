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
use std::cell::RefCell;

use actix_identity::Identity;
use actix_web::http::header;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use tera::Context;

use crate::api::v1::admin::campaigns::{runners, AddCapmaign};
use crate::errors::*;
use crate::AppData;

pub use super::*;

pub struct NewCampaign {
    ctx: RefCell<Context>,
}

pub const NEW_CAMPAIGN: TemplateFile =
    TemplateFile::new("new_campaign", "panel/campaigns/new/index.html");
pub const NEW_CAMPAIGN_FORM: TemplateFile =
    TemplateFile::new("new_campaign_form", "panel/campaigns/new/form.html");

impl CtxError for NewCampaign {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl NewCampaign {
    pub fn new(settings: &Settings) -> Self {
        let ctx = RefCell::new(context(settings, "Login"));
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(NEW_CAMPAIGN.name, &self.ctx.borrow())
            .unwrap()
    }
}

#[actix_web_codegen_const_routes::get(
    path = "PAGES.panel.campaigns.new",
    wrap = "crate::pages::get_page_check_login()"
)]
#[tracing::instrument(name = "New campaign form", skip(data))]
pub async fn new_campaign(data: AppData) -> PageResult<impl Responder, NewCampaign> {
    let new_campaign = NewCampaign::new(&data.settings).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(new_campaign))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormAddCampaign {
    pub name: String,
    pub difficulties: String,
}

impl FormAddCampaign {
    fn parse(self) -> ServiceResult<AddCapmaign> {
        let name = self.name;
        let mut difficulties = Vec::new();
        for d in self.difficulties.split(',') {
            let d = d.parse::<i32>().map_err(|_| ServiceError::NotANumber)?;
            difficulties.push(d);
        }
        Ok(AddCapmaign { name, difficulties })
    }
}

#[actix_web_codegen_const_routes::post(
    path = "PAGES.panel.campaigns.new",
    wrap = "crate::pages::get_page_check_login()"
)]
#[tracing::instrument(name = "New campaign form submit", skip(data, id))]
pub async fn new_campaign_submit(
    id: Identity,
    payload: web::Form<FormAddCampaign>,
    data: AppData,
) -> PageResult<impl Responder, NewCampaign> {
    let username = id.identity().unwrap();
    let mut payload = payload
        .into_inner()
        .parse()
        .map_err(|e| PageError::new(NewCampaign::new(&data.settings), e))?;

    runners::add_runner(&username, &mut payload, &data)
        .await
        .map_err(|e| PageError::new(NewCampaign::new(&data.settings), e))?;

    Ok(HttpResponse::Found()
        //TODO show stats of new campaign
        .insert_header((header::LOCATION, PAGES.panel.campaigns.home))
        .finish())
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(new_campaign);
    cfg.service(new_campaign_submit);
}

#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;

    use crate::tests::*;
    use crate::*;
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn new_campaign_form_works() {
        const NAME: &str = "testusercampaignform";
        const EMAIL: &str = "testcampaignuser@aaa.com";
        const PASSWORD: &str = "longpassword";

        const CAMPAIGN_NAME: &str = "testcampaignuser";

        let data = get_test_data().await;
        let app = get_app!(data).await;
        delete_user(NAME, &data).await;
        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);

        let mut difficulties = String::new();
        for d in DIFFICULTIES.iter() {
            if difficulties.is_empty() {
                difficulties = format!("{d}");
            } else {
                difficulties = format!("{difficulties},{d}");
            }
        }
        println!("{difficulties}");
        let new = super::FormAddCampaign {
            name: CAMPAIGN_NAME.into(),
            difficulties,
        };

        let new_resp = test::call_service(
            &app,
            post_request!(&new, crate::PAGES.panel.campaigns.new, FORM)
                .cookie(cookies)
                .to_request(),
        )
        .await;

        assert_eq!(new_resp.status(), StatusCode::FOUND);
        let headers = new_resp.headers();
        assert_eq!(
            headers.get(header::LOCATION).unwrap(),
            PAGES.panel.campaigns.home,
        );
    }
}
