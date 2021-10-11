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
use std::sync::Arc;

use actix_web::cookie::Cookie;
use actix_web::test;
use actix_web::{dev::ServiceResponse, error::ResponseError, http::StatusCode};
use serde::Serialize;

use super::*;
use crate::api::v1::admin::auth::runners::{Login, Register};
use crate::data::Data;
use crate::errors::*;
use crate::V1_API_ROUTES;

#[macro_export]
macro_rules! get_cookie {
    ($resp:expr) => {
        $resp.response().cookies().next().unwrap().to_owned()
    };
}

pub async fn delete_user(name: &str, data: &Data) {
    let r = sqlx::query!("DELETE FROM survey_admins WHERE name = ($1)", name,)
        .execute(&data.db)
        .await;
    println!();
    println!();
    println!();
    println!("Deleting user: {:?}", &r);
}

#[allow(dead_code, clippy::upper_case_acronyms)]
pub struct FORM;

#[macro_export]
macro_rules! post_request {
    ($uri:expr) => {
        test::TestRequest::post().uri($uri)
    };

    ($serializable:expr, $uri:expr) => {
        test::TestRequest::post()
            .uri($uri)
            .insert_header((actix_web::http::header::CONTENT_TYPE, "application/json"))
            .set_payload(serde_json::to_string($serializable).unwrap())
    };

    ($serializable:expr, $uri:expr, FORM) => {
        test::TestRequest::post().uri($uri).set_form($serializable)
    };
}

#[macro_export]
macro_rules! get_works {
    ($app:expr,$route:expr ) => {
        let list_sitekey_resp =
            test::call_service(&$app, test::TestRequest::get().uri($route).to_request())
                .await;
        assert_eq!(list_sitekey_resp.status(), StatusCode::OK);
    };
}

#[macro_export]
macro_rules! get_app {
    ("APP") => {
        actix_web::App::new()
            .app_data(crate::get_json_err())
            .wrap(crate::get_identity_service())
            .wrap(actix_web::middleware::NormalizePath::new(
                actix_web::middleware::TrailingSlash::Trim,
            ))
            .configure(crate::services)
    };

    () => {
        test::init_service(get_app!("APP"))
    };
    ($data:expr) => {
        test::init_service(
            get_app!("APP").app_data(actix_web::web::Data::new($data.clone())),
        )
    };
}

/// register and signin utility
pub async fn register_and_signin(
    name: &str,
    email: &str,
    password: &str,
) -> (Arc<data::Data>, Login, ServiceResponse) {
    register(name, email, password).await;
    signin(name, password).await
}

/// register utility
pub async fn register(name: &str, email: &str, password: &str) {
    let data = Data::new().await;
    let app = get_app!(data).await;

    // 1. Register
    let msg = Register {
        username: name.into(),
        password: password.into(),
        confirm_password: password.into(),
        email: Some(email.into()),
    };
    let resp = test::call_service(
        &app,
        post_request!(&msg, V1_API_ROUTES.admin.auth.register).to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::OK);
}

/// signin util
pub async fn signin(name: &str, password: &str) -> (Arc<Data>, Login, ServiceResponse) {
    let data = Data::new().await;
    let app = get_app!(data.clone()).await;

    // 2. signin
    let creds = Login {
        login: name.into(),
        password: password.into(),
    };
    let signin_resp = test::call_service(
        &app,
        post_request!(&creds, V1_API_ROUTES.admin.auth.login).to_request(),
    )
    .await;
    assert_eq!(signin_resp.status(), StatusCode::OK);
    (data, creds, signin_resp)
}

/// pub duplicate test
pub async fn bad_post_req_test<T: Serialize>(
    name: &str,
    password: &str,
    url: &str,
    payload: &T,
    err: ServiceError,
) {
    let (data, _, signin_resp) = signin(name, password).await;
    let cookies = get_cookie!(signin_resp);
    let app = get_app!(data).await;

    let resp = test::call_service(
        &app,
        post_request!(&payload, url)
            .cookie(cookies.clone())
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), err.status_code());
    let resp_err: ErrorToResponse = test::read_body_json(resp).await;
    //println!("{}", txt.error);
    assert_eq!(resp_err.error, format!("{}", err));
}
//
///// bad post req test without payload
//pub async fn bad_post_req_test_witout_payload(
//    name: &str,
//    password: &str,
//    url: &str,
//    err: ServiceError,
//) {
//    let (data, _, signin_resp) = signin(name, password).await;
//    let cookies = get_cookie!(signin_resp);
//    let app = get_app!(data).await;
//
//    let resp = test::call_service(
//        &app,
//        post_request!(url).cookie(cookies.clone()).to_request(),
//    )
//    .await;
//    assert_eq!(resp.status(), err.status_code());
//    let resp_err: ErrorToResponse = test::read_body_json(resp).await;
//    //println!("{}", txt.error);
//    assert_eq!(resp_err.error, format!("{}", err));
//}
//
//pub async fn create_new_campaign(
//    campaign_name: &str,
//    data: Arc<Data>,
//    cookies: Cookie<'_>,
//) -> CreateResp {
//    let new = CreateReq {
//        name: campaign_name.into(),
//    };
//
//    let app = get_app!(data).await;
//    let new_resp = test::call_service(
//        &app,
//        post_request!(&new, crate::V1_API_ROUTES.campaign.new)
//            .cookie(cookies)
//            .to_request(),
//    )
//    .await;
//    assert_eq!(new_resp.status(), StatusCode::OK);
//    let uuid: CreateResp = test::read_body_json(new_resp).await;
//    uuid
//}
//
//pub async fn delete_campaign(
//    camapign: &CreateResp,
//    data: Arc<Data>,
//    cookies: Cookie<'_>,
//) {
//    let del_route = V1_API_ROUTES.campaign.get_delete_route(&camapign.uuid);
//    let app = get_app!(data).await;
//    let del_resp =
//        test::call_service(&app, post_request!(&del_route).cookie(cookies).to_request())
//            .await;
//    assert_eq!(del_resp.status(), StatusCode::OK);
//}
//
//pub async fn list_campaings(
//    data: Arc<Data>,
//    cookies: Cookie<'_>,
//) -> Vec<ListCampaignResp> {
//    let app = get_app!(data).await;
//    let list_resp = test::call_service(
//        &app,
//        post_request!(crate::V1_API_ROUTES.campaign.list)
//            .cookie(cookies)
//            .to_request(),
//    )
//    .await;
//    assert_eq!(list_resp.status(), StatusCode::OK);
//    test::read_body_json(list_resp).await
//}
//
//pub async fn add_feedback(
//    rating: &RatingReq,
//    campaign: &CreateResp,
//    data: Arc<Data>,
//) -> RatingResp {
//    let add_feedback_route = V1_API_ROUTES.feedback.add_feedback_route(&campaign.uuid);
//    let app = get_app!(data).await;
//    let add_feedback_resp = test::call_service(
//        &app,
//        post_request!(&rating, &add_feedback_route).to_request(),
//    )
//    .await;
//    assert_eq!(add_feedback_resp.status(), StatusCode::OK);
//
//    test::read_body_json(add_feedback_resp).await
//}
//
//pub async fn get_feedback(
//    campaign: &CreateResp,
//    data: Arc<Data>,
//    cookies: Cookie<'_>,
//) -> GetFeedbackResp {
//    let get_feedback_route = V1_API_ROUTES.campaign.get_feedback_route(&campaign.uuid);
//    let app = get_app!(data).await;
//
//    let get_feedback_resp = test::call_service(
//        &app,
//        post_request!(&get_feedback_route)
//            .cookie(cookies)
//            .to_request(),
//    )
//    .await;
//    assert_eq!(get_feedback_resp.status(), StatusCode::OK);
//    test::read_body_json(get_feedback_resp).await
//}
