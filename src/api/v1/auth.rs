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
use actix_web::http::header;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use super::get_random;
use super::get_uuid;
use crate::errors::*;
use crate::AppData;

pub mod routes {
    pub struct Auth {
        pub register: &'static str,
    }

    impl Auth {
        pub const fn new() -> Auth {
            let register = "/api/v1/signup";
            Auth { register }
        }
    }
}

pub mod runners {
    //    use std::borrow::Cow;

    use super::*;

    pub async fn register_runner() -> ServiceResult<uuid::Uuid> {
        let mut uuid;

        loop {
            uuid = get_uuid();

            //            let res=    sqlx::query!(
            //                "INSERT INTO
            //                kaizen_feedbacks (helpful , description, uuid, campaign_id, time, page_url)
            //            VALUES ($1, $2, $3, $4, $5,
            //                 (SELECT ID from kaizen_campaign_pages WHERE page_url = $6))",
            //                &payload.helpful,
            //                &payload.description,
            //                &uuid,
            //                &campaign_id,
            //                &now,
            //                &payload.page_url,
            //            )
            //            .execute(&data.db)
            //            .await;
            //
            //            if res.is_ok() {
            //                break;
            //            } else if let Err(sqlx::Error::Database(err)) = res {
            //                if err.code() == Some(Cow::from("23505"))
            //                    && err.message().contains("kaizen_campaign_uuid_key")
            //                {
            //                    continue;
            //                } else {
            //                    return Err(sqlx::Error::Database(err).into());
            //                }
            //            }
            //        }
        }
        Ok(uuid)
    }
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(register);
}
#[my_codegen::post(path = "crate::V1_API_ROUTES.auth.register")]
async fn register(data: AppData, id: Identity) -> ServiceResult<impl Responder> {
    let uuid = runners::register_runner().await?;
    id.remember(uuid.to_string());
    Ok(HttpResponse::Ok())
}
