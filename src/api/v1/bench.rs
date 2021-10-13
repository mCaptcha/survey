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

use actix_identity::Identity;
use actix_web::{http, web, HttpResponse, Responder};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use uuid::Uuid;

use super::{get_uuid, RedirectQuery};
use crate::errors::*;
use crate::AppData;

pub mod routes {

    use crate::middleware::auth::GetLoginRoute;

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
            let submit = "/api/v1/benches/{campaign_id}/submit";
            let fetch = "/api/v1/benches/{campaign_id}/fetch";
            let register = "/api/v1/benches/register";
            let scope = "/api/v1/benches/";
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

    pub async fn register_runner(data: &AppData) -> ServiceResult<uuid::Uuid> {
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

#[my_codegen::get(path = "crate::V1_API_ROUTES.benches.register")]
async fn register(
    data: AppData,
    id: Identity,
    path: web::Query<RedirectQuery>,
) -> ServiceResult<HttpResponse> {
    let uuid = runners::register_runner(&data).await?;
    id.remember(uuid.to_string());
    let path = path.into_inner();
    if let Some(redirect_to) = path.redirect_to {
        Ok(HttpResponse::Found()
            .insert_header((http::header::LOCATION, redirect_to))
            .finish())
    } else {
        Ok(HttpResponse::Ok().into())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Bench {
    pub duration: f32,
    pub difficulty: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Submission {
    pub device_user_provided: String,
    pub device_software_recognised: String,
    pub threads: i32,
    pub benches: Vec<Bench>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SubmissionProof {
    pub token: String,
    pub proof: String,
}

fn get_check_login() -> crate::CheckLogin<routes::Benches> {
    crate::CheckLogin::new(crate::V1_API_ROUTES.benches)
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.benches.submit",
    wrap = "get_check_login()"
)]
async fn submit(
    data: AppData,
    id: Identity,
    payload: web::Json<Submission>,
    path: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let campaign_id = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;

    let user_id = Uuid::from_str(&username).unwrap();
    let payload = payload.into_inner();

    sqlx::query!(
        "INSERT INTO survey_responses (
                        user_id, 
                        campaign_id,
                        device_user_provided,
                        device_software_recognised,
                        threads
                    ) VALUES ($1, $2, $3, $4, $5);",
        &user_id,
        &campaign_id,
        &payload.device_user_provided,
        &payload.device_software_recognised,
        &payload.threads
    )
    .execute(&data.db)
    .await?;

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
        &user_id,
        &payload.device_software_recognised
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
        .execute(&data.db);

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

#[derive(Serialize, Deserialize)]
pub struct BenchConfig {
    pub difficulties: Vec<i32>,
}

#[my_codegen::post(
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
