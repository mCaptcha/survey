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
use std::str::FromStr;
use std::sync::Arc;

use actix_web::cookie::Cookie;
use actix_web::test;
use actix_web::{dev::ServiceResponse, error::ResponseError, http::StatusCode};
use serde::Serialize;
use uuid::Uuid;

use super::*;
use crate::api::v1::admin::{
    auth::runners::{Login, Register},
    campaigns::{AddCapmaign, AddCapmaignResp},
};
use crate::api::v1::bench::{BenchConfig, Submission, SubmissionProof};
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
        actix_web::test::init_service(
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

pub const DIFFICULTIES: [i32; 5] = [1, 2, 3, 4, 5];

pub async fn create_new_campaign(
    campaign_name: &str,
    data: Arc<Data>,
    cookies: Cookie<'_>,
) -> AddCapmaignResp {
    let new = AddCapmaign {
        name: campaign_name.into(),
        difficulties: DIFFICULTIES.into(),
    };

    let app = get_app!(data).await;
    let new_resp = test::call_service(
        &app,
        post_request!(&new, crate::V1_API_ROUTES.admin.campaign.add)
            .cookie(cookies)
            .to_request(),
    )
    .await;
    assert_eq!(new_resp.status(), StatusCode::OK);
    let uuid: AddCapmaignResp = test::read_body_json(new_resp).await;
    uuid
}

pub async fn get_survey_user(data: Arc<Data>) -> ServiceResponse {
    let app = get_app!(data).await;
    let signin_resp = test::call_service(
        &app,
        test::TestRequest::get()
            .uri(V1_API_ROUTES.benches.register)
            .to_request(),
    )
    .await;

    assert_eq!(signin_resp.status(), StatusCode::OK);
    signin_resp
}

pub async fn get_campaign_config(
    campaign: &AddCapmaignResp,
    data: Arc<Data>,
    cookies: Cookie<'_>,
) -> BenchConfig {
    let app = get_app!(data).await;
    let route = V1_API_ROUTES
        .benches
        .fetch_routes(&campaign.campaign_id.to_string());
    let new_resp =
        test::call_service(&app, post_request!(&route).cookie(cookies).to_request())
            .await;
    assert_eq!(new_resp.status(), StatusCode::OK);
    test::read_body_json(new_resp).await
}

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
pub async fn submit_bench(
    payload: &Submission,
    campaign: &AddCapmaignResp,
    cookies: Cookie<'_>,
    data: Arc<Data>,
) -> SubmissionProof {
    let route = V1_API_ROUTES.benches.submit_route(&campaign.campaign_id);
    let app = get_app!(data).await;
    let add_feedback_resp = test::call_service(
        &app,
        post_request!(&payload, &route).cookie(cookies).to_request(),
    )
    .await;
    assert_eq!(add_feedback_resp.status(), StatusCode::OK);

    let proof: SubmissionProof = test::read_body_json(add_feedback_resp).await;

    let survey_user_id = Uuid::from_str(&proof.token).unwrap();
    let proof_uuid = Uuid::from_str(&proof.proof).unwrap();

    struct Exists {
        exists: Option<bool>,
    }
    let res = sqlx::query_as!(
        Exists,
        "SELECT EXISTS (
                SELECT 1 from survey_responses 
                WHERE device_software_recognised = $1
                AND device_user_provided = $2
                AND THREADS = $3
                AND user_id = $4
                );",
        &payload.device_software_recognised,
        &payload.device_user_provided,
        payload.threads,
        &survey_user_id,
    )
    .fetch_one(&data.db)
    .await
    .unwrap();
    assert!(res.exists.as_ref().unwrap());

    let res = sqlx::query_as!(
        Exists,
        "SELECT EXISTS (
                SELECT 1 from survey_response_tokens 
                WHERE id = $1
                );",
        &proof_uuid
    )
    .fetch_one(&data.db)
    .await
    .unwrap();
    assert!(res.exists.as_ref().unwrap());

    struct ID {
        id: i32,
    }

    let resp_id = sqlx::query_as!(
        ID,
        "SELECT ID 
         FROM survey_responses 
         WHERE 
             user_id = $1 
         AND 
             device_software_recognised = $2;",
        &survey_user_id,
        &payload.device_software_recognised
    )
    .fetch_one(&data.db)
    .await
    .unwrap();

    for bench in payload.benches.iter() {
        let res = sqlx::query_as!(
            Exists,
            "SELECT EXISTS( 
            SELECT 1 FROM survey_benches 
            WHERE resp_id = $1
            AND difficulty = $2
            AND duration = $3);",
            &resp_id.id,
            &bench.difficulty,
            bench.duration
        )
        .fetch_one(&data.db)
        .await
        .unwrap();
        assert!(res.exists.as_ref().unwrap());
    }

    proof
}
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
