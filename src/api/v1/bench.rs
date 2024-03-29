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
use std::borrow::Cow;
use std::str::FromStr;

use actix_auth_middleware::*;
use actix_session::Session;
use actix_web::{dev::Payload, HttpRequest};
use actix_web::{http, web, HttpResponse, Responder};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;

use super::{get_uuid, RedirectQuery};
use crate::errors::*;
use crate::AppData;

pub const SURVEY_USER_ID: &str = "survey_user_id";

pub mod routes {
    use serde::{Deserialize, Serialize};

    use actix_auth_middleware::GetLoginRoute;

    #[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
    pub struct Benches {
        pub submit: &'static str,
        pub register: &'static str,
        pub fetch: &'static str,
        pub scope: &'static str,
    }

    impl GetLoginRoute for Benches {
        fn get_login_route(&self, src: Option<&str>) -> String {
            if let Some(redirect_to) = src {
                //                uri::Builder::new().path_and_query(
                format!(
                    "{}?redirect_to={}",
                    self.register,
                    urlencoding::encode(redirect_to)
                )
            //                let mut url: Uri = self.register.parse().unwrap();
            //                url.qu
            //                url.query_pairs_mut()
            //                    .append_pair("redirect_to", redirect_to);
            } else {
                self.register.to_string()
            }
        }
    }

    impl Benches {
        pub const fn new() -> Benches {
            let submit = "/survey/api/v1/benches/{campaign_id}/submit";
            let fetch = "/survey/api/v1/benches/{campaign_id}/fetch";
            let register = "/survey/api/v1/benches/register";
            let scope = "/survey/api/v1/benches/";
            Benches {
                submit,
                register,
                fetch,
                scope,
            }
        }
        pub fn submit_route(&self, campaign_id: &str) -> String {
            self.submit.replace("{campaign_id}", campaign_id)
        }
        pub fn fetch_routes(&self, campaign_id: &str) -> String {
            self.fetch.replace("{campaign_id}", campaign_id)
        }
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(submit);
    cfg.service(register);
    cfg.service(fetch);
}

pub mod runners {
    use super::*;

    pub async fn register_runner(data: &AppData) -> ServiceResult<sqlx::types::Uuid> {
        let mut uuid;
        let now = OffsetDateTime::now_utc();

        loop {
            uuid = get_uuid();

            let res = sqlx::query!(
                "
             INSERT INTO survey_users (created_at, id) VALUES($1, $2)",
                &now,
                &uuid
            )
            .execute(&data.db)
            .await;

            if res.is_ok() {
                break;
            } else if let Err(sqlx::Error::Database(err)) = res {
                if err.code() == Some(Cow::from("23505"))
                    && err.message().contains("survey_users_id_key")
                {
                    continue;
                } else {
                    return Err(sqlx::Error::Database(err).into());
                }
            }
        }
        Ok(uuid)
    }
}

#[actix_web_codegen_const_routes::get(path = "crate::V1_API_ROUTES.benches.register")]
async fn register(
    data: AppData,
    session: Session,
    path: web::Query<RedirectQuery>,
) -> ServiceResult<HttpResponse> {
    let is_authenticated = || {
        if let Ok(Some(_)) = session.get::<String>(SURVEY_USER_ID) {
            log::info!("user already registered");
            true
        } else {
            false
        }
    };

    if !is_authenticated() {
        let uuid = runners::register_runner(&data).await?;
        session.insert(SURVEY_USER_ID, uuid.to_string()).unwrap();
        session.get::<String>(SURVEY_USER_ID).unwrap().unwrap();
    }
    let path = path.into_inner();
    if let Some(redirect_to) = path.redirect_to {
        Ok(HttpResponse::Found()
            .insert_header((http::header::LOCATION, redirect_to))
            .finish())
    } else {
        Ok(HttpResponse::Ok().into())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bench {
    pub duration: f32,
    pub difficulty: i32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Submission {
    pub device_user_provided: String,
    pub device_software_recognised: String,
    pub threads: i32,
    pub benches: Vec<Bench>,
    pub submission_type: SubmissionType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SubmissionType {
    Wasm,
    Js,
}

impl ToString for SubmissionType {
    fn to_string(&self) -> String {
        let s = serde_json::to_string(&self).unwrap();
        (&s[1..(s.len() - 1)]).to_string()
    }
}

impl FromStr for SubmissionType {
    type Err = serde_json::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(&format!("\"{}\"", s))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmissionProof {
    pub token: String,
    pub proof: String,
}

fn is_session_authenticated(r: &HttpRequest, pl: &mut Payload) -> bool {
    use actix_web::FromRequest;
    matches!(
        Session::from_request(r, pl).into_inner().map(|x| {
            let val = x.get::<String>(SURVEY_USER_ID);
            val
        }),
        Ok(Ok(Some(_)))
    )
}

pub fn get_check_login() -> Authentication<routes::Benches> {
    Authentication::new(crate::V1_API_ROUTES.benches, is_session_authenticated)
}
//
// pub fn get_auth_middleware() -> Authentication<routes::Routes> {
//     Authentication::with_identity(V1_API_ROUTES)
// }
//}

#[actix_web_codegen_const_routes::post(
    path = "crate::V1_API_ROUTES.benches.submit",
    wrap = "get_check_login()"
)]
async fn submit(
    data: AppData,
    session: Session,
    payload: web::Json<Submission>,
    path: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let username = session.get::<String>(SURVEY_USER_ID).unwrap().unwrap();
    let path = path.into_inner();
    let campaign_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let user_id = Uuid::from_str(&username).unwrap();
    let payload = payload.into_inner();
    let now = OffsetDateTime::now_utc();

    struct ID {
        id: i32,
    }
    let resp_id = sqlx::query_as!(
        ID,
        "INSERT INTO survey_responses (
                    user_id,
                    campaign_id,
                    device_user_provided,
                    device_software_recognised,
                    threads,
                    submitted_at,
                    submission_bench_type_id
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    (SELECT ID FROM survey_bench_type WHERE name = $7)
                )
        RETURNING ID;",
        &user_id,
        &campaign_id,
        &payload.device_user_provided,
        &payload.device_software_recognised,
        &payload.threads,
        &now,
        &payload.submission_type.to_string(),
    )
    .fetch_one(&data.db)
    .await?;

    let mut futs = Vec::with_capacity(payload.benches.len());
    for bench in payload.benches.iter() {
        let fut = sqlx::query!(
            "INSERT INTO survey_benches 
                (resp_id, difficulty, duration) 
            VALUES ($1, $2, $3);",
            &resp_id.id,
            &bench.difficulty,
            bench.duration
        )
        .execute(&data.db); //.await?;
        futs.push(fut);
    }

    let mut submitions_id;
    try_join_all(futs).await?;

    loop {
        submitions_id = get_uuid();

        let res = sqlx::query!(
            "INSERT INTO survey_response_tokens 
            (resp_id, user_id, id)
            VALUES ($1, $2, $3);",
            &resp_id.id,
            &user_id,
            &submitions_id
        )
        .execute(&data.db)
        .await;

        if res.is_ok() {
            break;
        } else if let Err(sqlx::Error::Database(err)) = res {
            if err.code() == Some(Cow::from("23505"))
                && err.message().contains("survey_response_tokens_id_key")
            {
                continue;
            } else {
                return Err(sqlx::Error::Database(err).into());
            }
        }
    }

    let resp = SubmissionProof {
        token: username,
        proof: submitions_id.to_string(),
    };

    Ok(HttpResponse::Ok().json(resp))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchConfig {
    pub difficulties: Vec<i32>,
}

#[actix_web_codegen_const_routes::get(
    path = "crate::V1_API_ROUTES.benches.fetch",
    wrap = "get_check_login()"
)]
async fn fetch(data: AppData, path: web::Path<String>) -> ServiceResult<impl Responder> {
    let path = path.into_inner();
    let campaign_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let config = sqlx::query_as!(
        BenchConfig,
        "SELECT difficulties FROM survey_campaigns WHERE id = $1;",
        &campaign_id,
    )
    .fetch_one(&data.db)
    .await?;
    Ok(HttpResponse::Ok().json(config))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn survey_response_type_no_panic_test() {
        assert_eq!(SubmissionType::Wasm.to_string(), "wasm".to_string());
        assert_eq!(SubmissionType::Js.to_string(), "js".to_string());
    }
}
