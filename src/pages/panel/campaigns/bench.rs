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
use std::str::FromStr;

use actix_web::{web, HttpResponse, Responder};
use lazy_static::lazy_static;
use my_codegen::get;
use sailfish::TemplateOnce;
use uuid::Uuid;

use crate::errors::*;
use crate::pages::errors::ErrorPage;
use crate::PAGES;

#[derive(TemplateOnce)]
#[template(path = "bench/index.html")]
struct Bench<'a> {
    error: Option<ErrorPage<'a>>,
}
const PAGE: &str = "Survey";

impl<'a> Default for Bench<'a> {
    fn default() -> Self {
        Bench { error: None }
    }
}

lazy_static! {
    static ref BENCH: String = Bench::default().render_once().unwrap();
}

#[get(
    path = "PAGES.panel.campaigns.bench",
    wrap = "crate::api::v1::bench::get_check_login()"
)]
pub async fn bench(path: web::Path<String>) -> PageResult<impl Responder> {
    let path = path.into_inner();

    match Uuid::from_str(&path) {
        Err(_) => Err(PageError::PageDoesntExist),
        Ok(_) => Ok(HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(&*BENCH.as_str())),
    }
}
