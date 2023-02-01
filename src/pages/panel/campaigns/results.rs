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
use std::cell::RefCell;
use std::str::FromStr;

use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use tera::Context;
use uuid::Uuid;

use crate::api::v1::admin::campaigns::{runners, ResultsPage, SurveyResponse};
use crate::errors::ServiceError;
use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub struct CampaignResults {
    ctx: RefCell<Context>,
}

pub const CAMPAIGN_RESULTS: TemplateFile =
    TemplateFile::new("campaign_results", "panel/campaigns/results.html");

impl CtxError for CampaignResults {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

const RESUTS_LIMIT: usize = 10;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ResultsPagePayload {
    next_page: Option<String>,
    submissions: Vec<SurveyResponse>,
    pub wasm_only: Option<String>,
    pub js_only: Option<String>,
    pub all_benches: Option<String>,
}

impl ResultsPagePayload {
    pub fn new(
        submissions: Vec<SurveyResponse>,
        campaign_id: &Uuid,
        modifier: ResultsPage,
    ) -> Self {
        let campaign_id_str = campaign_id.to_string();

        let all_benches;
        let wasm_only;
        let js_only;
        match modifier.bench_type {
            Some(SubmissionType::Js) => {
                all_benches = Some(
                    crate::PAGES
                        .panel
                        .campaigns
                        .get_results_route(&campaign_id_str, None),
                );

                wasm_only = Some(crate::PAGES.panel.campaigns.get_results_route(
                    &campaign_id_str,
                    Some(ResultsPage::new(None, Some(SubmissionType::Wasm))),
                ));
                js_only = None;
            }
            Some(SubmissionType::Wasm) => {
                js_only = Some(crate::PAGES.panel.campaigns.get_results_route(
                    &campaign_id_str,
                    Some(ResultsPage::new(None, Some(SubmissionType::Js))),
                ));
                all_benches = Some(
                    crate::PAGES
                        .panel
                        .campaigns
                        .get_results_route(&campaign_id_str, None),
                );
                wasm_only = None;
            }
            None => {
                all_benches = None;
                js_only = Some(crate::PAGES.panel.campaigns.get_results_route(
                    &campaign_id_str,
                    Some(ResultsPage::new(None, Some(SubmissionType::Js))),
                ));
                wasm_only = Some(crate::PAGES.panel.campaigns.get_results_route(
                    &campaign_id_str,
                    Some(ResultsPage::new(None, Some(SubmissionType::Wasm))),
                ));
            }
        }

        let next_page = if submissions.len() == RESUTS_LIMIT {
            let m = ResultsPage::new(Some(modifier.page() + 1), modifier.bench_type);

            Some(
                PAGES
                    .panel
                    .campaigns
                    .get_results_route(&campaign_id_str, Some(m)),
            )
        } else {
            None
        };

        Self {
            next_page,
            submissions,
            js_only,
            wasm_only,
            all_benches,
        }
    }
}

impl CampaignResults {
    pub fn new(settings: &Settings, payload: Option<ResultsPagePayload>) -> Self {
        let ctx = RefCell::new(context(settings, "Results"));
        if let Some(payload) = payload {
            ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES
            .render(CAMPAIGN_RESULTS.name, &self.ctx.borrow())
            .unwrap()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.panel.campaigns.results")]
pub async fn results(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
    query: web::Query<ResultsPage>,
) -> PageResult<impl Responder, CampaignResults> {
    match Uuid::from_str(&path) {
        Err(_) => Err(PageError::new(
            CampaignResults::new(&data.settings, None),
            ServiceError::CampaignDoesntExist,
        )),
        Ok(uuid) => {
            let username = id.identity().unwrap();
            let query = query.into_inner();
            let page = query.page();

            let results = runners::get_results(
                &username,
                &uuid,
                &data,
                page,
                RESUTS_LIMIT,
                query.bench_type.clone(),
            )
            .await
            .map_err(|e| {
                PageError::new(CampaignResults::new(&data.settings, None), e)
            })?;
            let payload = ResultsPagePayload::new(results, &uuid, query);

            let results_page =
                CampaignResults::new(&data.settings, Some(payload)).render();
            let html = ContentType::html();
            Ok(HttpResponse::Ok().content_type(html).body(results_page))
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(results);
}
