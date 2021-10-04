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
use actix_web::HttpResponseBuilder;
use actix_web::{error::ResponseError, http::header, web, HttpResponse, Responder};
use lazy_static::lazy_static;
use my_codegen::{get, post};
use sailfish::TemplateOnce;

use crate::api::v1::campaign::{runners, CreateReq};
use crate::errors::*;
use crate::pages::errors::ErrorPage;
use crate::AppData;
use crate::PAGES;

#[derive(Clone, TemplateOnce)]
#[template(path = "panel/campaigns/new/index.html")]
struct NewCampaign<'a> {
    error: Option<ErrorPage<'a>>,
}

const PAGE: &str = "New Campaign";

impl<'a> Default for NewCampaign<'a> {
    fn default() -> Self {
        NewCampaign { error: None }
    }
}

impl<'a> NewCampaign<'a> {
    pub fn new(title: &'a str, message: &'a str) -> Self {
        Self {
            error: Some(ErrorPage::new(title, message)),
        }
    }
}

lazy_static! {
    static ref INDEX: String = NewCampaign::default().render_once().unwrap();
}

#[get(path = "PAGES.panel.campaigns.new", wrap = "crate::CheckLogin")]
pub async fn new_campaign() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(&*INDEX)
}

#[post(path = "PAGES.panel.campaigns.new", wrap = "crate::CheckLogin")]
pub async fn new_campaign_submit(
    id: Identity,
    payload: web::Form<CreateReq>,
    data: AppData,
) -> PageResult<impl Responder> {
    match runners::new(&payload.into_inner(), &data, &id).await {
        Ok(_) => {
            Ok(HttpResponse::Found()
                //TODO show stats of new campaign
                .insert_header((header::LOCATION, PAGES.panel.campaigns.home))
                .finish())
        }
        Err(e) => {
            let status = e.status_code();
            let heading = status.canonical_reason().unwrap_or("Error");

            Ok(HttpResponseBuilder::new(status)
                .content_type("text/html; charset=utf-8")
                .body(
                    NewCampaign::new(heading, &format!("{}", e))
                        .render_once()
                        .unwrap(),
                ))
        }
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;

    use crate::data::Data;
    use crate::tests::*;
    use crate::*;
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn new_campaign_form_works() {
        let data = Data::new().await;
        const NAME: &str = "testusercampaignform";
        const EMAIL: &str = "testcampaignuser@aaa.com";
        const PASSWORD: &str = "longpassword";

        const CAMPAIGN_NAME: &str = "testcampaignuser";

        let app = get_app!(data).await;
        delete_user(NAME, &data).await;
        let (_, _, signin_resp) = register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);

        let new = CreateReq {
            name: CAMPAIGN_NAME.into(),
        };

        let new_resp = test::call_service(
            &app,
            post_request!(&new, PAGES.panel.campaigns.new, FORM)
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
