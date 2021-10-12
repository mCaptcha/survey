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
        payload: &mut AddCapmaign,
        data: &AppData,
    ) -> ServiceResult<uuid::Uuid> {
        let mut uuid;
        let now = OffsetDateTime::now_utc();

        payload.difficulties.sort_unstable();

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
    pub name: String,
    pub difficulties: Vec<i32>,
}

#[derive(Serialize, Deserialize)]
pub struct AddCapmaignResp {
    pub campaign_id: String,
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
    let mut payload = payload.into_inner();
    let campaign_id = runners::add_runner(&username, &mut payload, &data).await?;
    let resp = AddCapmaignResp {
        campaign_id: campaign_id.to_string(),
    };
    Ok(HttpResponse::Ok().json(resp))
}

#[cfg(test)]
mod tests {
    use crate::api::v1::bench::Submission;
    use crate::data::Data;
    use crate::middleware::auth::GetLoginRoute;
    use crate::tests::*;
    use crate::*;

    use actix_web::{http::header, test};

    #[actix_rt::test]
    async fn test_bench_register_works() {
        let data = Data::new().await;
        let app = get_app!(data).await;
        let signin_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(V1_API_ROUTES.benches.register)
                .to_request(),
        )
        .await;

        assert_eq!(signin_resp.status(), StatusCode::OK);

        let redirect_to = Some("foo");

        let signin_resp = test::call_service(
            &app,
            test::TestRequest::get()
                .uri(&V1_API_ROUTES.benches.get_login_route(redirect_to))
                .to_request(),
        )
        .await;
        assert_eq!(signin_resp.status(), StatusCode::FOUND);
        let headers = signin_resp.headers();
        assert_eq!(
            headers.get(header::LOCATION).unwrap(),
            redirect_to.as_ref().unwrap()
        )
    }

    #[actix_rt::test]
    async fn test_add_campaign() {
        const NAME: &str = "testadminuser";
        const EMAIL: &str = "testuserupda@testadminuser.com";
        const PASSWORD: &str = "longpassword2";

        const DEVICE_USER_PROVIDED: &str = "foo";
        const DEVICE_SOFTWARE_RECOGNISED: &str = "Foobar.v2";
        const THREADS: i32 = 4;

        {
            let data = Data::new().await;
            delete_user(NAME, &data).await;
        }

        let (data, _creds, signin_resp) =
            register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let survey = get_survey_user(data.clone()).await;
        let survey_cookie = get_cookie!(survey);
        //        let app = get_app!(data).await;

        let campaign = create_new_campaign(NAME, data.clone(), cookies.clone()).await;
        let campaign_config =
            get_campaign_config(&campaign, data.clone(), survey_cookie.clone()).await;

        assert_eq!(DIFFICULTIES.to_vec(), campaign_config.difficulties);

        let submit_payload = Submission {
            device_user_provided: DEVICE_USER_PROVIDED.into(),
            device_software_recognised: DEVICE_SOFTWARE_RECOGNISED.into(),
            threads: THREADS,
            benches: BENCHES.clone(),
        };

        let _proof =
            submit_bench(&submit_payload, &campaign, survey_cookie, data.clone()).await;
    }
}
