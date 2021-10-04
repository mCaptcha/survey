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
use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    web, HttpResponse, Responder,
};
use my_codegen::{get, post};
use sailfish::TemplateOnce;
use uuid::Uuid;

use crate::api::v1::auth::runners::{login_runner, Login, Password};
use crate::api::v1::campaign::runners;
use crate::errors::*;
use crate::pages::auth::sudo::SudoPage;
use crate::AppData;
use crate::PAGES;

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
     FROM kaizen_campaign
     WHERE 
         uuid = $1
     AND
        user_id = (SELECT ID from kaizen_users WHERE name = $2)",
        &uuid,
        &username
    )
    .fetch_one(&data.db)
    .await?;

    Ok(format!("Delete camapign \"{}\"?", campaign.name))
}

#[get(path = "PAGES.panel.campaigns.delete", wrap = "crate::CheckLogin")]
pub async fn delete_campaign(
    id: Identity,
    path: web::Path<String>,
    data: AppData,
) -> PageResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let uuid = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let title = get_title(&username, &uuid, &data).await?;

    let page = SudoPage::new(
        &crate::PAGES.panel.campaigns.get_delete_route(&path),
        &title,
    )
    .render_once()
    .unwrap();
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page))
}

#[post(path = "PAGES.panel.campaigns.delete", wrap = "crate::CheckLogin")]
pub async fn delete_campaign_submit(
    id: Identity,
    path: web::Path<String>,
    payload: web::Form<Password>,
    data: AppData,
) -> PageResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let uuid = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;
    let payload = payload.into_inner();

    async fn render_err(
        e: ServiceError,
        username: &str,
        uuid: &Uuid,
        path: &str,
        data: &AppData,
    ) -> ServiceResult<(String, StatusCode)> {
        let status = e.status_code();
        let heading = status.canonical_reason().unwrap_or("Error");

        let form_route = crate::V1_API_ROUTES.campaign.get_delete_route(&path);
        let title = get_title(&username, &uuid, &data).await?;
        let mut ctx = SudoPage::new(&form_route, &title);
        let err = format!("{}", e);
        ctx.set_err(heading, &err);
        let page = ctx.render_once().unwrap();
        Ok((page, status))
    }

    let creds = Login {
        login: username,
        password: payload.password,
    };

    match login_runner(&creds, &data).await {
        Err(e) => {
            let (page, status) =
                render_err(e, &creds.login, &uuid, &path, &data).await?;

            Ok(HttpResponseBuilder::new(status)
                .content_type("text/html; charset=utf-8")
                .body(page))
        }
        Ok(_) => {
            match runners::delete(&uuid, &creds.login, &data).await {
                Ok(_) => Ok(HttpResponse::Found()
                    //TODO show stats of new campaign
                    .insert_header((header::LOCATION, PAGES.panel.campaigns.home))
                    .finish()),

                Err(e) => {
                    let (page, status) =
                        render_err(e, &creds.login, &uuid, &path, &data).await?;
                    Ok(HttpResponseBuilder::new(status)
                        .content_type("text/html; charset=utf-8")
                        .body(page))
                }
            }
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
        const NAME: &str = "delcappageuser";
        const EMAIL: &str = "delcappageuser@aaa.com";
        const PASSWORD: &str = "longpassword";
        const CAMPAIGN_NAME: &str = "delcappageusercamaping";

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
                &PAGES.panel.campaigns.get_delete_route(&uuid.uuid),
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
