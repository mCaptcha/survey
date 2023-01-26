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

use actix_web::http::header::ContentType;
use actix_web::{web, HttpResponse, Responder};
use tera::Context;
use uuid::Uuid;

use crate::AppData;
use crate::PAGES;

pub use super::*;

pub struct Bench {
    ctx: RefCell<Context>,
}

pub const BENCH: TemplateFile = TemplateFile::new("new_bench", "bench/index.html");

impl CtxError for Bench {
    fn with_error(&self, e: &ReadableError) -> String {
        self.ctx.borrow_mut().insert(ERROR_KEY, e);
        self.render()
    }
}

impl Bench {
    pub fn new(settings: &Settings) -> Self {
        let ctx = RefCell::new(context(settings, "Login"));
        Self { ctx }
    }

    pub fn render(&self) -> String {
        TEMPLATES.render(BENCH.name, &self.ctx.borrow()).unwrap()
    }
}

#[actix_web_codegen_const_routes::get(
    path = "PAGES.panel.campaigns.bench",
    wrap = "crate::api::v1::bench::get_check_login()"
)]
pub async fn bench(
    data: AppData,
    _path: web::Path<Uuid>,
) -> PageResult<impl Responder, Bench> {
    let bench = Bench::new(&data.settings).render();
    let html = ContentType::html();
    Ok(HttpResponse::Ok().content_type(html).body(bench))
}

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(bench);
}
