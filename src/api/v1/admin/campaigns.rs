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
use uuid::Uuid;

use super::{get_admin_check_login, get_uuid};
use crate::errors::*;
use crate::AppData;

pub mod routes {
    pub struct Campaign {
        pub add: &'static str,
        pub delete: &'static str,
        //    pub get_feedback: &'static str,
        pub list: &'static str,
    }

    impl Campaign {
        pub const fn new() -> Campaign {
            let add = "/admin/api/v1/campaign/add";
            let delete = "/admin/api/v1/campaign/{uuid}/delete";
            //            let get_feedback = "/api/v1/campaign/{uuid}/feedback";
            let list = "/admin/api/v1/campaign/list";

            Campaign { add, delete, list }
        }
        //        pub fn get_benches_route(&self, campaign_id: &str) -> String {
        //            self.get_feedback.replace("{uuid}", &campaign_id)
        //        }

        pub fn get_delete_route(&self, campaign_id: &str) -> String {
            self.delete.replace("{uuid}", campaign_id)
        }
    }
}

pub mod runners {
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

    pub async fn list_campaign_runner(
        username: &str,
        data: &AppData,
    ) -> ServiceResult<Vec<ListCampaignResp>> {
        struct ListCampaign {
            name: String,
            id: Uuid,
        }

        let mut campaigns = sqlx::query_as!(
            ListCampaign,
            "SELECT 
            name, id
        FROM 
            survey_campaigns 
            WHERE
                user_id = (
                    SELECT 
                        ID
                    FROM 
                        survey_admins
                    WHERE
                        name = $1
                )",
            username
        )
        .fetch_all(&data.db)
        .await?;

        let mut list_resp = Vec::with_capacity(campaigns.len());
        campaigns.drain(0..).for_each(|c| {
            list_resp.push(ListCampaignResp {
                name: c.name,
                uuid: c.id.to_string(),
            });
        });

        Ok(list_resp)
    }

    //    pub async fn get_benches(
    //        username: &str,
    //        uuid: &str,
    //        data: &AppData,
    //    ) -> ServiceResult<GetFeedbackResp> {
    //        let uuid = Uuid::parse_str(uuid).map_err(|_| ServiceError::NotAnId)?;
    //
    //        struct FeedbackInternal {
    //            time: OffsetDateTime,
    //            description: String,
    //            helpful: bool,
    //        }
    //
    //        struct Name {
    //            name: String,
    //        }
    //
    //        let name_fut = sqlx::query_as!(
    //            Name,
    //            "SELECT name
    //            FROM survey_campaigns
    //            WHERE uuid = $1
    //            AND
    //                user_id = (
    //                    SELECT
    //                        ID
    //                   FROM
    //                        kaizen_users
    //                    WHERE
    //                        name = $2
    //                )
    //           ",
    //            uuid,
    //            username
    //        )
    //        .fetch_one(&data.db); //.await?;
    //
    //        let feedback_fut = sqlx::query_as!(
    //            FeedbackInternal,
    //            "SELECT
    //            time, description, helpful
    //        FROM
    //            kaizen_feedbacks
    //        WHERE campaign_id = (
    //            SELECT uuid
    //            FROM
    //                survey_campaigns
    //            WHERE
    //                uuid = $1
    //            AND
    //                user_id = (
    //                    SELECT
    //                        ID
    //                    FROM
    //                        kaizen_users
    //                    WHERE
    //                        name = $2
    //                )
    //           )",
    //            uuid,
    //            username
    //        )
    //        .fetch_all(&data.db);
    //        let (name, mut feedbacks) = try_join!(name_fut, feedback_fut)?;
    //        //.await?;
    //
    //        let mut feedback_resp = Vec::with_capacity(feedbacks.len());
    //        feedbacks.drain(0..).for_each(|f| {
    //            feedback_resp.push(Feedback {
    //                time: f.time.unix_timestamp() as u64,
    //                description: f.description,
    //                helpful: f.helpful,
    //            });
    //        });
    //
    //        Ok(GetFeedbackResp {
    //            feedbacks: feedback_resp,
    //            name: name.name,
    //        })
    //    }

    pub async fn delete(
        uuid: &Uuid,
        username: &str,
        data: &AppData,
    ) -> ServiceResult<()> {
        sqlx::query!(
            "DELETE 
            FROM survey_campaigns 
         WHERE 
             user_id = (
                 SELECT 
                         ID 
                 FROM 
                         survey_admins 
                 WHERE 
                         name = $1
             )
         AND
            id = ($2)",
            username,
            uuid
        )
        .execute(&data.db)
        .await?;
        Ok(())
    }
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.admin.campaign.delete",
    wrap = "get_admin_check_login()"
)]
pub async fn delete(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let uuid = Uuid::parse_str(&path).map_err(|_| ServiceError::NotAnId)?;
    runners::delete(&uuid, &username, &data).await?;
    Ok(HttpResponse::Ok())
}

//#[derive(Serialize, Deserialize)]
//pub struct Feedback {
//    pub time: u64,
//    pub description: String,
//    pub helpful: bool,
//}
//
//#[derive(Serialize, Deserialize)]
//pub struct GetFeedbackResp {
//    pub name: String,
//    pub feedbacks: Vec<Feedback>,
//}
//
//#[my_codegen::post(
//    path = "crate::V1_API_ROUTES.campaign.get_feedback",
//    wrap = "crate::CheckLogin"
//)]
//pub async fn get_feedback(
//    id: Identity,
//    data: AppData,
//    path: web::Path<String>,
//) -> ServiceResult<impl Responder> {
//    let username = id.identity().unwrap();
//    let path = path.into_inner();
//    let feedback_resp = runners::get_feedback(&username, &path, &data).await?;
//    Ok(HttpResponse::Ok().json(feedback_resp))
//}

#[derive(Serialize, Deserialize)]
pub struct ListCampaignResp {
    pub name: String,
    pub uuid: String,
}

#[my_codegen::post(
    path = "crate::V1_API_ROUTES.admin.campaign.list",
    wrap = "get_admin_check_login()"
)]
pub async fn list_campaign(
    id: Identity,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let list_resp = runners::list_campaign_runner(&username, &data).await?;

    Ok(HttpResponse::Ok().json(list_resp))
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
    cfg.service(delete);
    cfg.service(list_campaign);
    //cfg.service(get_feedback);
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
    use crate::errors::*;
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

        let list = list_campaings(data.clone(), cookies.clone()).await;
        assert!(list.iter().any(|c| c.name == NAME));

        bad_post_req_test_witout_payload(
            NAME,
            PASSWORD,
            &V1_API_ROUTES.admin.campaign.delete.replace("{uuid}", NAME),
            ServiceError::NotAnId,
        )
        .await;

        delete_campaign(&campaign, data.clone(), cookies.clone()).await;

        let list = list_campaings(data.clone(), cookies.clone()).await;
        assert!(!list.iter().any(|c| c.name == NAME));
    }
}
