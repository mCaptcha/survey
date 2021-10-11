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

use actix_identity::Identity;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;

use super::get_uuid;
use crate::errors::*;
use crate::AppData;

pub mod routes {
    pub struct Campaign {
        pub add: &'static str,
    }

    impl Campaign {
        pub const fn new() -> Campaign {
            let add = "/api/v1/admin/campaign/add";
            Campaign { add }
        }
    }
}

pub mod runners {
    //    use std::borrow::Cow;

    use super::*;

    pub async fn add_runner(
        username: &str,
        payload: &AddCapmaign,
        data: &AppData,
    ) -> ServiceResult<uuid::Uuid> {
        let mut uuid;
        let now = OffsetDateTime::now_utc();

        loop {
            uuid = get_uuid();

            let res = sqlx::query!(
                "
INSERT INTO survey_campaigns (
    user_id, ID, name, difficulties, created_at
    ) VALUES(
        (SELECT id FROM survey_admins WHERE name = $1),
        $2, $3, $4, $5
    );",
                username,
                &uuid,
                &payload.name,
                &payload.difficulties,
                &now
            )
            .execute(&data.db)
            .await;

            if res.is_ok() {
                break;
            } else if let Err(sqlx::Error::Database(err)) = res {
                if err.code() == Some(Cow::from("23505"))
                    && err.message().contains("survey_admins_id_key")
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

#[derive(Serialize, Deserialize)]
pub struct AddCapmaign {
    name: String,
    difficulties: Vec<i32>,
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(add);
}
#[my_codegen::post(path = "crate::V1_API_ROUTES.admin.campaign.add")]
async fn add(
    payload: web::Json<AddCapmaign>,
    data: AppData,
    id: Identity,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let payload = payload.into_inner();
    let _campaign_id = runners::add_runner(&username, &payload, &data).await?;
    Ok(HttpResponse::Ok())
}
