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
use std::cell::RefCell;

use actix_identity::Identity;
use actix_web::http::header::{self, ContentType};
use tera::Context;

use crate::api::v1::admin::auth::runners;
use crate::api::v1::RedirectQuery;
use crate::pages::errors::*;
use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub struct Login {
    ctx: RefCell<Context>,
}

pub const LOGIN: TemplateFile = TemplateFile::new("login", "auth/login/index.html");

impl CtxError for Login {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Login {
    pub fn new(settings: &Settings) -> Self {
        let ctx = RefCell::new(context(settings, "Login"));
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(LOGIN.name, &self.ctx.borrow()).unwrap()
    }

    pub fn page(s: &Settings) -> String {
        let p = Self::new(s);
        p.render()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.auth.login")]
#[tracing::instrument(name = "Serve login page", skip(ctx))]
pub async fn get_login(ctx: AppData) -> impl Responder {
    let login = Login::page(&ctx.settings);
    let html = ContentType::html();
    HttpResponse::Ok().content_type(html).body(login)
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(get_login);
    cfg.service(login_submit);
}

#[actix_web_codegen_const_routes::post(path = "PAGES.auth.login")]
#[tracing::instrument(name = "Web UI Login", skip(id, payload, data, path))]
pub async fn login_submit(
    id: Identity,
    payload: web::Form<runners::Login>,
    data: AppData,
    path: web::Path<RedirectQuery>,
) -> PageResult<impl Responder, Login> {
    let payload = payload.into_inner();
    let username = runners::login_runner(&payload, &data)
        .await
        .map_err(|e| PageError::new(Login::new(&data.settings), e))?;

    id.remember(username);
    let path = path.into_inner();
    if let Some(redirect_to) = path.redirect_to {
        Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, redirect_to))
            .finish())
    } else {
        Ok(HttpResponse::Found()
            .insert_header((header::LOCATION, PAGES.home))
            .finish())
    }
}
#[cfg(test)]
mod tests {
    use actix_web::test;

    use super::*;

    use crate::api::v1::admin::auth::runners::{Login, Register};
    use crate::tests::*;
    use crate::*;
    use actix_web::http::StatusCode;

    #[actix_rt::test]
    async fn auth_form_works() {
        let data = get_test_data().await;
        const NAME: &str = "testuserform";
        const PASSWORD: &str = "longpassword";

        let app = get_app!(data).await;

        delete_user(NAME, &data).await;

        // 1. Register with email == None
        let msg = Register {
            username: NAME.into(),
            password: PASSWORD.into(),
            confirm_password: PASSWORD.into(),
            email: None,
        };
        let resp = test::call_service(
            &app,
            post_request!(&msg, V1_API_ROUTES.admin.auth.register).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);

        // correct form login
        let msg = Login {
            login: NAME.into(),
            password: PASSWORD.into(),
        };

        let resp = test::call_service(
            &app,
            post_request!(&msg, PAGES.auth.login, FORM).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::FOUND);
        let headers = resp.headers();
        assert_eq!(headers.get(header::LOCATION).unwrap(), PAGES.home,);

        // incorrect form login
        let msg = Login {
            login: NAME.into(),
            password: NAME.into(),
        };
        let resp = test::call_service(
            &app,
            post_request!(&msg, PAGES.auth.login, FORM).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // non-existent form login
        let msg = Login {
            login: PASSWORD.into(),
            password: PASSWORD.into(),
        };
        let resp = test::call_service(
            &app,
            post_request!(&msg, PAGES.auth.login, FORM).to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
