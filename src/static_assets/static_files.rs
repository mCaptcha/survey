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

use actix_web::body::BoxBody;
use actix_web::{get, http::header, web, HttpResponse, Responder};
use log::debug;
use mime_guess::from_path;
use rust_embed::RustEmbed;

use crate::CACHE_AGE;

pub mod assets {
    use serde::*;

    #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Img {
        pub path: &'static str,
        pub name: &'static str,
    }
}

#[derive(RustEmbed)]
#[folder = "assets/"]
struct Asset;

fn handle_assets(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => {
            let body: BoxBody = match content.data {
                Cow::Borrowed(bytes) => BoxBody::new(bytes),
                Cow::Owned(bytes) => BoxBody::new(bytes),
            };

            HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![
                    header::CacheDirective::Public,
                    header::CacheDirective::Extension("immutable".into(), None),
                    header::CacheDirective::MaxAge(CACHE_AGE),
                ]))
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[get("/assets/{_:.*}")]
pub async fn static_files(path: web::Path<String>) -> impl Responder {
    handle_assets(&path)
}

#[derive(RustEmbed)]
#[folder = "static/favicons/"]
struct Favicons;

fn handle_favicons(path: &str) -> HttpResponse {
    match Favicons::get(path) {
        Some(content) => {
            let body: BoxBody = match content.data {
                Cow::Borrowed(bytes) => BoxBody::new(bytes),
                Cow::Owned(bytes) => BoxBody::new(bytes),
            };

            HttpResponse::Ok()
                .insert_header(header::CacheControl(vec![
                    header::CacheDirective::Public,
                    header::CacheDirective::Extension("immutable".into(), None),
                    header::CacheDirective::MaxAge(CACHE_AGE),
                ]))
                .content_type(from_path(path).first_or_octet_stream().as_ref())
                .body(body)
        }
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[get("/{file}")]
pub async fn favicons(path: web::Path<String>) -> impl Responder {
    debug!("searching favicons");
    handle_favicons(&path)
}

#[cfg(test)]
mod tests {
    use actix_web::http::StatusCode;
    use actix_web::test;

    use super::*;
    use crate::static_assets::routes::ASSETS;
    use crate::tests::get_test_data;
    use crate::*;

    #[actix_rt::test]
    async fn static_assets_work() {
        let data = get_test_data().await;
        let app = get_app!(data).await;

        for file in [
            ASSETS.logo.path,
            ASSETS.js,
            ASSETS.css,
            ASSETS.mobile_css,
            ASSETS.glue,
        ]
        .iter()
        {
            let resp = test::call_service(
                &app,
                test::TestRequest::get().uri(file).to_request(),
            )
            .await;
            assert_eq!(resp.status(), StatusCode::OK);
        }
    }

    #[actix_rt::test]
    async fn favicons_work() {
        assert!(Favicons::get("favicon.ico").is_some());

        let data = get_test_data().await;
        let app = get_app!(data).await;

        let resp = test::call_service(
            &app,
            test::TestRequest::get().uri("/favicon.ico").to_request(),
        )
        .await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}
