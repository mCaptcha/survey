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
use tera::Context;
use uuid::Uuid;

use crate::errors::ServiceError;
use crate::settings::Settings;
use crate::AppData;

pub use super::*;

pub struct Intro {
    ctx: RefCell<Context>,
}

pub const INTRO: TemplateFile = TemplateFile::new("intro", "index.html");

impl CtxError for Intro {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Intro {
    pub fn new(settings: &Settings, payload: Option<&str>) -> Self {
        let ctx = RefCell::new(context(settings, "Campaign Homepage"));
        if let Some(uuid) = payload {
            let payload = crate::PAGES.panel.campaigns.get_bench_route(uuid);
            ctx.borrow_mut().insert(PAYLOAD_KEY, &payload);
        }
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(INTRO.name, &self.ctx.borrow()).unwrap()
    }
}

#[actix_web_codegen_const_routes::get(path = "PAGES.panel.campaigns.about")]
pub async fn about(
    data: AppData,
    path: web::Path<String>,
) -> PageResult<impl Responder, Intro> {
    let path = path.into_inner();

    match Uuid::from_str(&path) {
        Err(_) => Err(PageError::new(
            Intro::new(&data.settings, None),
            ServiceError::CampaignDoesntExist,
        )),
        Ok(_) => {
            let about = Intro::new(&data.settings, Some(&path)).render();
            let html = ContentType::html();
            Ok(HttpResponse::Ok().content_type(html).body(about))
        }
    }
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(about);
}
