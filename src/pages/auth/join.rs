/*
 * Copyright (C) 2022  Aravinth Manivannan <realaravinth@batsense.net>
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
use actix_web::http::header::ContentType;
use std::cell::RefCell;
use tera::Context;

use crate::pages::errors::*;
use crate::settings::Settings;

use actix_web::{http::header, web, HttpResponse, Responder};

use crate::api::v1::admin::auth::runners;
use crate::AppData;
use crate::PAGES;

pub use super::*;

pub const REGISTER: TemplateFile = TemplateFile::new("register", "auth/join/index.html");

pub struct Register {
    ctx: RefCell<Context>,
}

impl CtxError for Register {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Register {
    fn new(settings: &Settings) -> Self {
        let ctx = RefCell::new(context(settings, "Join"));
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(REGISTER.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s);
        p.render()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.auth.join")]
#[tracing::instrument(name = "Serve registration page", skip(ctx))]
pub async fn get_join(ctx: AppData) -> impl Responder {
    let login = Register::page(&ctx.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_join);
    cfg.service(join_submit);
}

#[actix_web_codegen_const_routes::post(path = "PAGES.auth.join")]
#[tracing::instrument(name = "Process web UI registration", skip(data))]
pub async fn join_submit(
    payload: web::Form<runners::Register>,
    data: AppData,
) -> PageResult<impl Responder, Register> {
    let mut payload = payload.into_inner();
    if payload.email.is_some() && payload.email.as_ref().unwrap().is_empty() {
        payload.email = None;
    }

    runners::register_runner(&payload, &data)
        .await
        .map_err(|e| PageError::new(Register::new(&data.settings), e))?;

    Ok(HttpResponse::Found()
        .insert_header((header::LOCATION, PAGES.auth.login))
        .finish())
}

#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;

    use crate::api::v1::admin::account::{
        username::runners::username_exists, AccountCheckPayload,
    };
    use crate::api::v1::admin::auth::runners::Register;
    use crate::data::Data;
    use crate::tests::*;
    use crate::*;
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn auth_join_form_works() {
        let settings = Settings::new().unwrap();
        let data = Data::new(settings).await;
        const NAME: &str = "testuserformjoin";
        const NAME2: &str = "testuserformjoin2";
        const EMAIL: &str = "testuserformjoin@a.com";
        const PASSWORD: &str = "longpassword";

        let app = get_app!(data).await;

        delete_user(NAME, &data).await;

        // 1. Register with email == None
        let mut msg = Register {
            username: NAME.into(),
            password: PASSWORD.into(),
            confirm_password: PASSWORD.into(),
            email: Some(EMAIL.into()),
        };

        let resp = test::call_service(
            &app,
            post_request!(&msg, PAGES.auth.join, FORM).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let headers = resp.headers();
        assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.auth.login,);

        let account_check = AccountCheckPayload { val: NAME.into() };
        assert!(
            username_exists(&account_check, &AppData::new(data.clone()))
                .await
                .unwrap()
                .exists
        );

        msg.email = None;
        let resp = test::call_service(
            &app,
            post_request!(&msg, PAGES.auth.join, FORM).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

        msg.email = Some(EMAIL.into());
        msg.username = NAME2.into();
        let resp = test::call_service(
            &app,
            post_request!(&msg, PAGES.auth.join, FORM).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }
}
