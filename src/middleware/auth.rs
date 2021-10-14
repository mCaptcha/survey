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
#![allow(clippy::type_complexity)]

use std::rc::Rc;

use crate::api::v1::bench::SURVEY_USER_ID;
use actix_http::body::AnyBody;
use actix_identity::Identity;
use actix_service::{Service, Transform};
use actix_session::Session;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::{http, Error, FromRequest, HttpResponse};

#[derive(Clone)]
pub enum AuthenticatedSession {
    ActixIdentity,
    ActixSession,
}

use futures::future::{ok, Either, Ready};

pub trait GetLoginRoute {
    fn get_login_route(&self, src: Option<&str>) -> String;
}

pub struct CheckLogin<T: GetLoginRoute> {
    login: Rc<T>,
    session_type: AuthenticatedSession,
}

impl<T: GetLoginRoute> CheckLogin<T> {
    pub fn new(login: T, session_type: AuthenticatedSession) -> Self {
        let login = Rc::new(login);
        Self {
            login,
            session_type,
        }
    }
}

impl<S, GT> Transform<S, ServiceRequest> for CheckLogin<GT>
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error>,
    S::Future: 'static,
    GT: GetLoginRoute,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    type Transform = CheckLoginMiddleware<S, GT>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(CheckLoginMiddleware {
            service,
            login: self.login.clone(),
            session_type: self.session_type.clone(),
        })
    }
}
pub struct CheckLoginMiddleware<S, GT> {
    service: S,
    login: Rc<GT>,
    session_type: AuthenticatedSession,
}

impl<S, GT> Service<ServiceRequest> for CheckLoginMiddleware<S, GT>
where
    S: Service<ServiceRequest, Response = ServiceResponse<AnyBody>, Error = Error>,
    S::Future: 'static,
    GT: GetLoginRoute,
{
    type Response = ServiceResponse<AnyBody>;
    type Error = Error;
    type Future = Either<S::Future, Ready<Result<Self::Response, Self::Error>>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let (r, mut pl) = req.into_parts();
        let mut is_authenticated = || match self.session_type {
            AuthenticatedSession::ActixSession => {
                if let Ok(Ok(Some(_))) = Session::from_request(&r, &mut pl)
                    .into_inner()
                    .map(|x| x.get::<String>(SURVEY_USER_ID))
                {
                    true
                } else {
                    false
                }
            }

            AuthenticatedSession::ActixIdentity => {
                if let Ok(Some(_)) = Identity::from_request(&r, &mut pl)
                    .into_inner()
                    .map(|x| x.identity())
                {
                    true
                } else {
                    false
                }
            }
        };
        if is_authenticated() {
            let req = ServiceRequest::from_parts(r, pl);
            Either::Left(self.service.call(req))
        } else {
            let path = r.uri().path_and_query().map(|path| path.as_str());
            let path = self.login.get_login_route(path);
            let req = ServiceRequest::from_parts(r, pl);
            Either::Right(ok(req.into_response(
                HttpResponse::Found()
                    .insert_header((http::header::LOCATION, path))
                    .finish(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::api::v1::bench::Submission;
    use crate::data::Data;
    use crate::middleware::auth::GetLoginRoute;
    use crate::tests::*;
    use crate::*;

    use actix_web::{http::header, test};

    #[actix_rt::test]
    async fn auth_middleware_works() {
        fn make_uri(path: &str, queries: &Option<Vec<(&str, &str)>>) -> String {
            let mut url = Url::parse("http://x/").unwrap();
            let final_path;
            url.set_path(path);

            if let Some(queries) = queries {
                {
                    let mut query_pairs = url.query_pairs_mut();
                    queries.iter().for_each(|(k, v)| {
                        query_pairs.append_pair(k, v);
                    });
                }

                final_path = format!("{}?{}", url.path(), url.query().unwrap());
            } else {
                final_path = url.path().to_string();
            }
            final_path
        }

        const NAME: &str = "testmiddlewareuser";
        const EMAIL: &str = "testuserupda@testmiddlewareuser.com";
        const PASSWORD: &str = "longpassword2";
        const DEVICE_USER_PROVIDED: &str = "foo";
        const DEVICE_SOFTWARE_RECOGNISED: &str = "Foobar.v2";
        const THREADS: i32 = 4;
        let queries = Some(vec![
            ("foo", "bar"),
            ("src", "/x/y/z"),
            ("with_q", "/a/b/c/?goo=x"),
        ]);

        {
            let data = Data::new().await;
            delete_user(NAME, &data).await;
        }
        let (data, _creds, signin_resp) =
            register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let survey = get_survey_user(data.clone()).await;
        let survey_cookie = get_cookie!(survey);

        let campaign = create_new_campaign(NAME, data.clone(), cookies.clone()).await;

        let bench_submit_route =
            V1_API_ROUTES.benches.submit_route(&campaign.campaign_id);
        let bench_routes = vec![
            (&bench_submit_route, queries.clone()),
            (&bench_submit_route, None),
        ];

        let app = get_app!(data).await;

        //    let campaign_routes = vec![
        //        (Some(V1_API_ROUTES.camp.submit), queries.clone()),
        //        (None, None),
        //        (Some(V1_API_ROUTES.benches.submit), None),
        //    ];

        let bench_submit_payload = Submission {
            device_user_provided: DEVICE_USER_PROVIDED.into(),
            device_software_recognised: DEVICE_SOFTWARE_RECOGNISED.into(),
            threads: THREADS,
            benches: BENCHES.clone(),
        };

        for (from, query) in bench_routes.iter() {
            let route = make_uri(from, query);
            let signin_resp = test::call_service(
                &app,
                post_request!(&bench_submit_payload, &route).to_request(),
            )
            .await;
            assert_eq!(signin_resp.status(), StatusCode::FOUND);

            let redirect_to = V1_API_ROUTES.benches.get_login_route(Some(&route));
            let headers = signin_resp.headers();
            assert_eq!(headers.get(header::LOCATION).unwrap(), &redirect_to);

            let add_feedback_resp = test::call_service(
                &app,
                post_request!(&bench_submit_payload, &route)
                    .cookie(survey_cookie.clone())
                    .to_request(),
            )
            .await;
            assert_eq!(add_feedback_resp.status(), StatusCode::OK);
        }
    }

    //    let signin_resp = test::call_service(
    //        &app,
    //        test::TestRequest::get()
    //            .uri(V1_API_ROUTES.benches.get_login_route(redirect_to).as_ref().unwrap())
    //            .to_request(),
    //    )
    //    .await;
    //    assert_eq!(signin_resp.status(), StatusCode::FOUND);
    //    let headers = signin_resp.headers();
    //    assert_eq!(
    //        headers.get(header::LOCATION).unwrap(),
    //        redirect_to.as_ref().unwrap()
    //    )
    //
}
