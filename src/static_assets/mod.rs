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
pub mod filemap;
pub mod static_files;

pub use filemap::FileMap;

pub fn services(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(static_files::static_files);
    cfg.service(static_files::favicons);
}

pub mod routes {
    use lazy_static::lazy_static;
    use serde::*;

    use super::static_files::assets::Img;
    use crate::FILES;

    lazy_static! {
        pub static ref ASSETS: Assets = Assets::new();
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
    pub struct Svg {
        pub trash: Img,
    }

    impl Svg {
        /// create new instance of Routes
        fn new() -> Svg {
            let trash = Img {
                path: FILES.get("./static/cache/img/trash.svg").unwrap(),
                name: "Trash icon",
            };

            Svg { trash }
        }
    }

    #[derive(Clone, Debug, PartialEq, Eq, Serialize)]
    /// Top-level routes data structure for V1 AP1
    pub struct Assets {
        /// Authentication routes
        pub css: &'static str,
        pub mobile_css: &'static str,
        pub js: &'static str,
        pub glue: &'static str,
        pub logo: Img,
        pub svg: Svg,
    }

    impl Assets {
        /// create new instance of Routes
        pub fn new() -> Assets {
            let logo = Img {
                path: FILES.get("./static/cache/img/icon-trans.png").unwrap(),
                name: "mCaptcha logo",
            };

            Assets {
                css: *crate::CSS,
                mobile_css: *crate::MOBILE_CSS,
                js: *crate::JS,
                glue: *crate::GLUE,
                svg: Svg::new(),
                logo,
            }
        }
    }
}
