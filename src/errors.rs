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
use std::convert::From;

use actix::MailboxError;
use actix_web::{
    error::ResponseError,
    http::{header, StatusCode},
    HttpResponse, HttpResponseBuilder,
};
use derive_more::{Display, Error};
use libmcaptcha::errors::CaptchaError;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot::error::RecvError;
use url::ParseError;
use validator::ValidationErrors;

#[derive(Debug, Display, PartialEq, Error)]
#[cfg(not(tarpaulin_include))]
pub enum ServiceError {
    #[display(fmt = "internal server error")]
    InternalServerError,

    /// when the a username is already taken
    #[display(fmt = "Username not available")]
    UsernameTaken,

    /// email is already taken
    #[display(fmt = "Email not available")]
    EmailTaken,

    /// when the a token name is already taken
    /// token not found
    #[display(fmt = "Token not found. Is token registered?")]
    TokenNotFound,

    #[display(fmt = "{}", _0)]
    CaptchaError(CaptchaError),
}

#[derive(Serialize, Deserialize)]
#[cfg(not(tarpaulin_include))]
pub struct ErrorToResponse {
    pub error: String,
}

#[cfg(not(tarpaulin_include))]
impl ResponseError for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .append_header((header::CONTENT_TYPE, "application/json; charset=UTF-8"))
            .body(
                serde_json::to_string(&ErrorToResponse {
                    error: self.to_string(),
                })
                .unwrap(),
            )
            .into()
    }

    #[cfg(not(tarpaulin_include))]
    fn status_code(&self) -> StatusCode {
        match self {
            ServiceError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,

            ServiceError::UsernameTaken => StatusCode::BAD_REQUEST,
            ServiceError::EmailTaken => StatusCode::BAD_REQUEST,

            ServiceError::TokenNotFound => StatusCode::NOT_FOUND,
            ServiceError::CaptchaError(e) => {
                log::error!("{}", e);
                match e {
                    CaptchaError::MailboxError => StatusCode::INTERNAL_SERVER_ERROR,
                    _ => StatusCode::BAD_REQUEST,
                }
            }
        }
    }
}

#[cfg(not(tarpaulin_include))]
impl From<CaptchaError> for ServiceError {
    fn from(e: CaptchaError) -> ServiceError {
        ServiceError::CaptchaError(e)
    }
}

#[cfg(not(tarpaulin_include))]
impl From<sqlx::Error> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: sqlx::Error) -> Self {
        // use sqlx::error::Error;
        // use std::borrow::Cow;

        ServiceError::InternalServerError
    }
}

#[cfg(not(tarpaulin_include))]
impl From<RecvError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: RecvError) -> Self {
        log::error!("{:?}", e);
        ServiceError::InternalServerError
    }
}

#[cfg(not(tarpaulin_include))]
impl From<MailboxError> for ServiceError {
    #[cfg(not(tarpaulin_include))]
    fn from(e: MailboxError) -> Self {
        log::error!("{:?}", e);
        ServiceError::InternalServerError
    }
}

#[cfg(not(tarpaulin_include))]
pub type ServiceResult<V> = std::result::Result<V, ServiceError>;

//#[derive(Debug, Display, PartialEq, Error)]
//#[cfg(not(tarpaulin_include))]
//pub enum PageError {
//    #[display(fmt = "Something weng wrong: Internal server error")]
//    InternalServerError,
//
//    #[display(fmt = "{}", _0)]
//    ServiceError(ServiceError),
//}
//
//#[cfg(not(tarpaulin_include))]
//impl From<sqlx::Error> for PageError {
//    #[cfg(not(tarpaulin_include))]
//    fn from(_: sqlx::Error) -> Self {
//        PageError::InternalServerError
//    }
//}
//
//#[cfg(not(tarpaulin_include))]
//impl From<ServiceError> for PageError {
//    #[cfg(not(tarpaulin_include))]
//    fn from(e: ServiceError) -> Self {
//        PageError::ServiceError(e)
//    }
//}
//
//impl ResponseError for PageError {
//    fn error_response(&self) -> HttpResponse {
//        use crate::PAGES;
//        match self.status_code() {
//            StatusCode::INTERNAL_SERVER_ERROR => HttpResponse::Found()
//                .append_header((header::LOCATION, PAGES.errors.internal_server_error))
//                .finish(),
//            _ => HttpResponse::Found()
//                .append_header((header::LOCATION, PAGES.errors.unknown_error))
//                .finish(),
//        }
//    }
//
//    #[cfg(not(tarpaulin_include))]
//    fn status_code(&self) -> StatusCode {
//        match self {
//            PageError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
//            PageError::ServiceError(e) => e.status_code(),
//        }
//    }
//}
//
//#[cfg(not(tarpaulin_include))]
//pub type PageResult<V> = std::result::Result<V, PageError>;
//
//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::PAGES;
//
//    #[test]
//    fn error_works() {
//        let resp: HttpResponse = PageError::InternalServerError.error_response();
//        assert_eq!(resp.status(), StatusCode::FOUND);
//        let headers = resp.headers();
//        assert_eq!(
//            headers.get(header::LOCATION).unwrap(),
//            PAGES.errors.internal_server_error
//        );
//    }
//}
