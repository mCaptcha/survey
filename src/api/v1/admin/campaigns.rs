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
use sqlx::types::Uuid;

use super::{get_admin_check_login, get_uuid};
use crate::api::v1::bench::Bench;
use crate::api::v1::bench::SubmissionType;
use crate::errors::*;
use crate::AppData;

pub mod routes {
    use serde::{Deserialize, Serialize};

    use super::ResultsPage;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
    pub struct Campaign {
        pub add: &'static str,
        pub delete: &'static str,
        //    pub get_feedback: &'static str,
        pub list: &'static str,
        pub results: &'static str,
    }

    impl Campaign {
        pub const fn new() -> Campaign {
            let add = "/admin/api/v1/campaign/add";
            let delete = "/admin/api/v1/campaign/{uuid}/delete";
            //            let get_feedback = "/api/v1/campaign/{uuid}/feedback";
            let list = "/admin/api/v1/campaign/list";
            let results = "/admin/api/v1/campaign/{uuid}/results";

            Campaign {
                add,
                delete,
                list,
                results,
            }
        }
        //        pub fn get_benches_route(&self, campaign_id: &str) -> String {
        //            self.get_feedback.replace("{uuid}", &campaign_id)
        //        }

        pub fn get_delete_route(&self, campaign_id: &str) -> String {
            self.delete.replace("{uuid}", campaign_id)
        }

        pub fn get_results_route(
            &self,
            campaign_id: &str,
            modifier: Option<ResultsPage>,
        ) -> String {
            let mut res = self.results.replace("{uuid}", campaign_id);
            if let Some(modifier) = modifier {
                if let Some(page) = modifier.page {
                    res = format!("{res}?page={page}");
                }

                if let Some(bench_type) = modifier.bench_type {
                    if modifier.page.is_some() {
                        res = format!("{res}&bench_type={}", bench_type.to_string());
                    } else {
                        res = format!("{res}?bench_type={}", bench_type.to_string());
                    }
                }
            }
            res
        }
    }
}

pub mod runners {
    use std::str::FromStr;

    use futures::try_join;

    use crate::api::v1::bench::Bench;

    use super::*;

    pub async fn add_runner(
        username: &str,
        payload: &mut AddCapmaign,
        data: &AppData,
    ) -> ServiceResult<sqlx::types::Uuid> {
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

    #[derive(Debug)]
    struct InternalSurveyResp {
        id: i32,
        submitted_at: OffsetDateTime,
        user_id: Uuid,
        threads: Option<i32>,
        device_user_provided: String,
        device_software_recognised: String,
        name: String,
    }

    #[derive(Debug)]
    struct InnerU {
        created_at: OffsetDateTime,
        id: Uuid,
    }

    impl From<InnerU> for SurveyUser {
        fn from(u: InnerU) -> Self {
            Self {
                id: uuid::Uuid::parse_str(&u.id.to_string()).unwrap(),
                created_at: u.created_at.unix_timestamp(),
            }
        }
    }

    pub async fn get_results(
        username: &str,
        uuid: &Uuid,
        data: &AppData,
        page: usize,
        limit: usize,
        filter: Option<SubmissionType>,
    ) -> ServiceResult<Vec<SurveyResponse>> {
        let mut db_responses = if let Some(filter) = filter {
            sqlx::query_as!(
                InternalSurveyResp,
                "SELECT
                    survey_responses.ID,
                    survey_responses.device_software_recognised,
                    survey_responses.threads,
                    survey_responses.user_id,
                    survey_responses.submitted_at,
                    survey_responses.device_user_provided,
                    survey_bench_type.name
                FROM
                    survey_responses
                INNER JOIN  survey_bench_type ON
                    survey_responses.submission_bench_type_id = survey_bench_type.ID
                WHERE
                    survey_bench_type.name = $3
                AND
                    survey_responses.campaign_id = (
                        SELECT ID FROM survey_campaigns
                        WHERE
                            ID = $1
                        AND
                            user_id = (SELECT ID FROM survey_admins WHERE name = $2)
                    )
                LIMIT $4 OFFSET $5",
                uuid,
                username,
                filter.to_string(),
                limit as i32,
                page as i32,
            )
            .fetch_all(&data.db)
            .await?
        } else {
            #[derive(Debug)]
            struct I {
                id: i32,
                submitted_at: OffsetDateTime,
                user_id: Uuid,
                threads: Option<i32>,
                device_user_provided: String,
                device_software_recognised: String,
                name: String,
            }
            let mut i = sqlx::query_as!(
                I,
                "SELECT
                survey_responses.ID,
                survey_responses.device_software_recognised,
                survey_responses.threads,
                survey_responses.user_id,
                survey_responses.submitted_at,
                survey_responses.device_user_provided,
                survey_bench_type.name
            FROM
                survey_responses
            INNER JOIN  survey_bench_type ON
                survey_responses.submission_bench_type_id = survey_bench_type.ID
            WHERE
                survey_responses.campaign_id = (
                    SELECT ID FROM survey_campaigns
                    WHERE
                        ID = $1
                    AND
                        user_id = (SELECT ID FROM survey_admins WHERE name = $2)
                )
            LIMIT $3 OFFSET $4",
                uuid,
                username,
                limit as i32,
                page as i32,
            )
            .fetch_all(&data.db)
            .await?;

            let mut res = Vec::with_capacity(i.len());
            i.drain(0..).for_each(|x| {
                res.push(InternalSurveyResp {
                    id: x.id,
                    submitted_at: x.submitted_at,
                    user_id: x.user_id,
                    threads: x.threads,
                    device_user_provided: x.device_user_provided,
                    device_software_recognised: x.device_software_recognised,
                    name: x.name,
                })
            });
            res
        };

        let mut responses = Vec::with_capacity(db_responses.len());
        for r in db_responses.drain(0..) {
            let benches_fut = sqlx::query_as!(
                Bench,
                "SELECT
                    duration,
                    difficulty
                FROM
                    survey_benches
                WHERE
                    resp_id = $1
               ",
                r.id,
            )
            .fetch_all(&data.db);

            let user_fut = sqlx::query_as!(
                InnerU,
                "SELECT
                    created_at,
                    ID
                FROM
                    survey_users
                WHERE
                    ID = $1
               ",
                r.user_id,
            )
            .fetch_one(&data.db);

            let (benches, user) = try_join!(benches_fut, user_fut)?;
            let user = user.into();
            responses.push(SurveyResponse {
                benches,
                user,
                device_user_provided: r.device_user_provided,
                device_software_recognised: r.device_software_recognised,
                submitted_at: r.submitted_at.unix_timestamp(),
                id: r.id as usize,
                submission_type: SubmissionType::from_str(&r.name).unwrap(),
                threads: r.threads.map(|t| t as usize),
            })
        }
        Ok(responses)
    }

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

#[actix_web_codegen_const_routes::post(
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyResponse {
    pub user: SurveyUser,
    pub device_user_provided: String,
    pub device_software_recognised: String,
    pub id: usize,
    pub threads: Option<usize>,
    pub submitted_at: i64,
    pub submission_type: SubmissionType,
    pub benches: Vec<Bench>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyUser {
    pub created_at: i64, // OffsetDateTime,
    pub id: uuid::Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListCampaignResp {
    pub name: String,
    pub uuid: String,
}

#[actix_web_codegen_const_routes::post(
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddCapmaign {
    pub name: String,
    pub difficulties: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AddCapmaignResp {
    pub campaign_id: String,
}

pub fn services(cfg: &mut web::ServiceConfig) {
    cfg.service(add);
    cfg.service(delete);
    cfg.service(list_campaign);
    cfg.service(get_campaign_resutls);
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ResultsPage {
    page: Option<usize>,
    pub bench_type: Option<SubmissionType>,
}

impl ResultsPage {
    pub fn page(&self) -> usize {
        self.page.unwrap_or(0)
    }

    pub fn new(page: Option<usize>, bench_type: Option<SubmissionType>) -> Self {
        Self { page, bench_type }
    }
}

#[actix_web_codegen_const_routes::get(
    path = "crate::V1_API_ROUTES.admin.campaign.results",
    wrap = "get_admin_check_login()"
)]
pub async fn get_campaign_resutls(
    id: Identity,
    query: web::Query<ResultsPage>,
    path: web::Path<uuid::Uuid>,
    data: AppData,
) -> ServiceResult<impl Responder> {
    let username = id.identity().unwrap();
    let query = query.into_inner();
    let page = query.page();
    let path = Uuid::parse_str(&path.to_string()).unwrap();

    let results =
        runners::get_results(&username, &path, &data, page, 50, query.bench_type)
            .await?;

    Ok(HttpResponse::Ok().json(results))
}

#[actix_web_codegen_const_routes::post(path = "crate::V1_API_ROUTES.admin.campaign.add")]
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
    use crate::api::v1::bench::SubmissionType;
    use crate::errors::*;
    use crate::tests::*;
    use crate::*;

    use actix_auth_middleware::GetLoginRoute;
    use actix_web::{http::header, test};

    #[actix_rt::test]
    async fn test_bench_register_works() {
        let data = get_test_data().await;
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
            let data = get_test_data().await;
            delete_user(NAME, &data).await;
        }

        let (data, _creds, signin_resp) =
            register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let survey = get_survey_user(data.clone()).await;
        let survey_cookie = get_cookie!(survey);
        let app = get_app!(data).await;

        let campaign = create_new_campaign(NAME, data.clone(), cookies.clone()).await;
        let campaign_config =
            get_campaign_config(&campaign, data.clone(), survey_cookie.clone()).await;

        assert_eq!(DIFFICULTIES.to_vec(), campaign_config.difficulties);

        let submit_payload = Submission {
            device_user_provided: DEVICE_USER_PROVIDED.into(),
            device_software_recognised: DEVICE_SOFTWARE_RECOGNISED.into(),
            threads: THREADS,
            benches: BENCHES.clone(),
            submission_type: SubmissionType::Wasm,
        };

        let _proof =
            submit_bench(&submit_payload, &campaign, survey_cookie, data.clone()).await;

        let list = list_campaings(data.clone(), cookies.clone()).await;
        assert!(list.iter().any(|c| c.name == NAME));

        let responses = super::runners::get_results(
            NAME,
            &sqlx::types::Uuid::parse_str(&campaign.campaign_id).unwrap(),
            &AppData::new(data.clone()),
            0,
            50,
            None,
        )
        .await
        .unwrap();
        assert_eq!(responses.len(), 1);
        assert_eq!(responses[0].threads, Some(THREADS as usize));
        let mut l = responses[0].benches.clone();
        l.sort_by(|a, b| a.difficulty.cmp(&b.difficulty));
        let mut r = BENCHES.clone();
        r.sort_by(|a, b| a.difficulty.cmp(&b.difficulty));

        assert_eq!(
            super::runners::get_results(
                NAME,
                &sqlx::types::Uuid::parse_str(&campaign.campaign_id).unwrap(),
                &AppData::new(data.clone()),
                0,
                50,
                Some(SubmissionType::Wasm),
            )
            .await
            .unwrap(),
            responses
        );

        assert_eq!(
            super::runners::get_results(
                NAME,
                &sqlx::types::Uuid::parse_str(&campaign.campaign_id).unwrap(),
                &AppData::new(data.clone()),
                0,
                50,
                Some(SubmissionType::Js),
            )
            .await
            .unwrap(),
            Vec::default()
        );

        assert_eq!(l, r);
        assert_eq!(
            responses[0].device_software_recognised,
            DEVICE_SOFTWARE_RECOGNISED
        );
        assert_eq!(responses[0].device_user_provided, DEVICE_USER_PROVIDED);

        let results_resp = get_request!(
            &app,
            &V1_API_ROUTES
                .admin
                .campaign
                .get_results_route(&campaign.campaign_id, None),
            cookies.clone()
        );
        assert_eq!(results_resp.status(), StatusCode::OK);
        let res: Vec<super::SurveyResponse> = test::read_body_json(results_resp).await;
        assert_eq!(responses, res);

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
