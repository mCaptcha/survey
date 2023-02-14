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
use actix_identity::Identity;
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;

use crate::api::v1::admin::auth::runners::{login_runner, Login, Password};
use crate::api::v1::admin::campaigns::runners;
use crate::errors::*;
//use crate::AppData;
//use crate::PAGES;

use std::cell::RefCell;

use actix_web::http::header;
use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use tera::Context;

//use crate::api::v1::admin::campaigns::{runners, AddCapmaign};
use crate::AppData;

pub use super::*;

pub struct SudoDelete {
    ctx: RefCell<Context>,
}

pub const SUDO_DELETE: TemplateFile =
    TemplateFile::new("sudo_delete_campaign", "panel/campaigns/delete/index.html");

impl CtxError for SudoDelete {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl SudoDelete {
    pub fn new(settings: &Settings, payload: Option<CampaignDeletePayload>) -> Self {
        let ctx = RefCell::new(context(settings, "Login"));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(SUDO_DELETE.name, &self.ctx.borrow())
            .unwrap()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct CampaignDeletePayload {
    pub title: String,
    pub delete_url: String,
}

async fn get_title(
    username: &str,
    uuid: &Uuid,
    data: &AppData,
) -> ServiceResult<String> {
    struct Name {
        name: String,
    }
    let campaign = sqlx::query_as!(
        Name,
        "SELECT name 
     FROM survey_campaigns
     WHERE 
         id = $1
     AND
        user_id = (SELECT ID from survey_admins WHERE name = $2)",
        &uuid,
        &username
    )
    .fetch_one(&data.db)
    .await?;

    Ok(format!("Delete camapign \"{}\"?", campaign.name))
}

#[actix_web_codegen_const_routes::get(
    path = "PAGES.panel.campaigns.delete",
    wrap = "crate::pages::get_page_check_login()"
)]
pub async fn delete_campaign(
    id: Identity,
    path: web::Path<uuid::Uuid>,
    data: AppData,
) -> PageResult<impl Responder, SudoDelete> {
    let username = id.identity().unwrap();
    let uuid = Uuid::parse_str(&path.to_string()).unwrap();

    let title = get_title(&username, &uuid, &data)
        .await
        .map_err(|e| PageError::new(SudoDelete::new(&data.settings, None), e))?;
    let delete_url = crate::PAGES
        .panel
        .campaigns
        .get_delete_route(&uuid.to_string());

    let payload = CampaignDeletePayload { title, delete_url };

    let page = SudoDelete::new(&data.settings, Some(payload)).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(page))
}

#[actix_web_codegen_const_routes::post(
    path = "PAGES.panel.campaigns.delete",
    wrap = "crate::pages::get_page_check_login()"
)]
pub async fn delete_campaign_submit(
    id: Identity,
    uuid: web::Path<uuid::Uuid>,
    payload: web::Form<Password>,
    data: AppData,
) -> PageResult<impl Responder, SudoDelete> {
    let username = id.identity().unwrap();
    let payload = payload.into_inner();
    let uuid = Uuid::parse_str(&uuid.to_string()).unwrap();

    let creds = Login {
        login: username,
        password: payload.password,
    };

    login_runner(&creds, &data)
        .await
        .map_err(|e| PageError::new(SudoDelete::new(&data.settings, None), e))?;
    runners::delete(&uuid, &creds.login, &data)
        .await
        .map_err(|e| PageError::new(SudoDelete::new(&data.settings, None), e))?;

    Ok(HttpResponse::Found()
        //TODO show stats of new campaign
        .insert_header((header::LOCATION, PAGES.panel.campaigns.home))
        .finish())
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(delete_campaign);
    cfg.service(delete_campaign_submit);
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
        const NAME: &str = "delcappageuser";
        const EMAIL: &str = "delcappageuser@aaa.com";
        const PASSWORD: &str = "longpassword";
        const CAMPAIGN_NAME: &str = "delcappageusercamaping";

        let data = get_test_data().await;
        let app = get_app!(data).await;
        delete_user(NAME, &data).await;
        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);

        let uuid =
            create_new_campaign(CAMPAIGN_NAME, data.clone(), cookies.clone()).await;

        let creds = Password {
            password: PASSWORD.into(),
        };

        let new_resp = test::call_service(
            &app,
            post_request!(
                &creds,
                &PAGES.panel.campaigns.get_delete_route(&uuid.campaign_id),
                FORM
            )
            .cookie(cookies.clone())
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
