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
use actix_web::{web, HttpResponse, Responder};
use my_codegen::get;
use sailfish::TemplateOnce;

use crate::api::v1::campaign::{runners, GetFeedbackResp};
use crate::AppData;
use crate::PAGES;

#[derive(TemplateOnce)]
#[template(path = "panel/campaigns/get/index.html")]
struct ViewFeedback<'a> {
    campaign: GetFeedbackResp,
    uuid: &'a str,
}

const PAGE: &str = "New Campaign";

impl<'a> ViewFeedback<'a> {
    pub fn new(campaign: GetFeedbackResp, uuid: &'a str) -> Self {
        Self { campaign, uuid }
    }
}

#[get(
    path = "PAGES.panel.campaigns.get_feedback",
    wrap = "crate::CheckLogin"
)]
pub async fn get_feedback(
    id: Identity,
    data: AppData,
    path: web::Path<String>,
) -> impl Responder {
    let username = id.identity().unwrap();
    let path = path.into_inner();
    let feedback_resp = runners::get_feedback(&username, &path, &data)
        .await
        .unwrap();
    let page = ViewFeedback::new(feedback_resp, &path)
        .render_once()
        .unwrap();
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(page)
}
