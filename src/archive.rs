use std::collections::HashMap;
/*
 * Copyright (C) 2023  Aravinth Manivannan <realaravinth@batsense.net>
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
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::api::v1::admin::campaigns::runners::get_results;
use crate::api::v1::admin::campaigns::SurveyResponse;
use crate::{errors::ServiceResult, AppData, Settings};

const CAMPAIGN_INFO_FILE: &str = "campaign.json";

const BENCHMARK_FILE: &str = "benchmark.csv";

pub struct Archiver {
    base_path: String,
}

impl Archiver {
    pub fn new(s: &Settings) -> Self {
        Archiver {
            base_path: s.archive.base_path.clone(),
        }
    }
    fn campaign_path(&self, id: &Uuid) -> PathBuf {
        Path::new(&self.base_path).join(&id.to_string())
    }

    async fn create_dir_util(p: &PathBuf) -> ServiceResult<()> {
        if p.exists() {
            if !p.is_dir() {
                fs::remove_file(&p).await.unwrap();
                fs::create_dir_all(&p).await.unwrap();
            }
        } else {
            fs::create_dir_all(&p).await.unwrap();
        }
        Ok(())
    }

    fn archive_path_now(&self, id: &Uuid) -> PathBuf {
        let unix_time = OffsetDateTime::now_utc().unix_timestamp();
        self.campaign_path(id).join(unix_time.to_string())
    }

    fn campaign_file_path(&self, id: &Uuid) -> PathBuf {
        self.archive_path_now(id).join(CAMPAIGN_INFO_FILE)
    }

    fn benchmark_file_path(&self, id: &Uuid) -> PathBuf {
        self.archive_path_now(id).join(BENCHMARK_FILE)
    }

    async fn write_campaign_file(&self, c: &Campaign) -> ServiceResult<()> {
        let archive_path = self.archive_path_now(&c.id);
        Self::create_dir_util(&archive_path).await?;
        let campaign_file_path = self.campaign_file_path(&c.id);
        let contents = serde_json::to_string(c).unwrap();
        //        fs::write(campaign_file_path, contents).await.unwrap();
        let mut file = fs::File::create(&campaign_file_path).await.unwrap();
        file.write(contents.as_bytes()).await.unwrap();
        file.flush().await.unwrap();

        Ok(())
    }

    async fn write_benchmark_file(
        &self,
        c: &Campaign,
        data: &AppData,
    ) -> ServiceResult<()> {
        let archive_path = self.archive_path_now(&c.id);
        Self::create_dir_util(&archive_path).await?;

        let benchmark_file_path = self.benchmark_file_path(&c.id);
        struct Username {
            name: String,
        }
        let owner = sqlx::query_as!(
            Username,
            "SELECT
                survey_admins.name
            FROM
                survey_admins
            INNER JOIN survey_campaigns ON
                survey_admins.ID = survey_campaigns.user_id
            WHERE
                survey_campaigns.ID = $1
            ",
            &c.id
        )
        .fetch_one(&data.db)
        .await?;

        let mut page = 0;
        let limit = 50;
        let file = fs::OpenOptions::new()
            .read(true)
            .append(true)
            .create(true)
            .open(&benchmark_file_path)
            .await
            .unwrap();
        let mut wri = csv_async::AsyncWriter::from_writer(file);

        loop {
            let mut resp =
                get_results(&owner.name, &c.id, data, page, limit, None).await?;

            for r in resp.drain(0..) {
                let csv_resp = to_hashmap(r, c);
                let keys: Vec<&str> = csv_resp.keys().map(|k| k.as_str()).collect();
                wri.write_record(&keys).await.unwrap();

                let values: Vec<&str> = csv_resp.values().map(|v| v.as_str()).collect();
                wri.write_record(&values).await.unwrap();
                wri.flush().await.unwrap();

                //wri.serialize(csv_resp).await.unwrap();
                wri.flush().await.unwrap();
            }
            page += 1;

            wri.flush().await.unwrap();
            if resp.len() < limit {
                break;
            }
        }

        Ok(())
    }

    pub async fn archive(&self, data: &AppData) -> ServiceResult<()> {
        let mut db_campaigns = sqlx::query_as!(
            InnerCampaign,
            "SELECT ID, name, difficulties, created_at FROM survey_campaigns"
        )
        .fetch_all(&data.db)
        .await?;
        for c in db_campaigns.drain(0..) {
            let campaign: Campaign = c.into();
            self.write_campaign_file(&campaign).await?;
            self.write_benchmark_file(&campaign, data).await?;
        }
        Ok(())
    }
}

pub fn to_hashmap(s: SurveyResponse, c: &Campaign) -> HashMap<String, String> {
    let mut map = HashMap::with_capacity(7 + c.difficulties.len());
    map.insert("user".into(), s.user.id.to_string());
    map.insert("device_user_provided".into(), s.device_user_provided);
    map.insert(
        "device_software_recognised".into(),
        s.device_software_recognised,
    );
    map.insert(
        "threads".into(),
        s.threads.map_or_else(|| "-".into(), |v| v.to_string()),
    );
    map.insert("submitted_at".into(), s.submitted_at.to_string());
    map.insert("submission_type".into(), s.submission_type.to_string());
    for d in c.difficulties.iter() {
        let bench = s
            .benches
            .iter()
            .find(|b| b.difficulty == *d as i32)
            .map_or_else(|| "-".into(), |v| v.duration.to_string());
        map.insert(format!("Difficulty: {d}"), bench);
    }
    map
}

//#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
//pub struct CSVSurveyResp {
//    pub user: Uuid,
//    pub device_user_provided: String,
//    pub device_software_recognised: String,
//    pub id: usize,
//    pub threads: Option<usize>,
//    pub submitted_at: i64,
//    pub submission_type: SubmissionType,
//    pub benches: String,
//}
//
//impl From<SurveyResponse> for CSVSurveyResp {
//    fn from(s: SurveyResponse) -> Self {
//        let mut benches = String::default();
//        for b in s.benches.iter() {
//            benches = format!("{benches} ({})", b.to_csv_resp());
//        }
//        Self {
//            user: s.user.id,
//            device_software_recognised: s.device_software_recognised,
//            device_user_provided: s.device_user_provided,
//            id: s.id,
//            threads: s.threads,
//            submission_type: s.submission_type,
//            benches,
//            submitted_at: s.submitted_at,
//        }
//    }
//}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerCampaign {
    id: Uuid,
    name: String,
    difficulties: Vec<i32>,
    created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    pub id: Uuid,
    pub name: String,
    pub difficulties: Vec<u32>,
    pub created_at: i64,
}

impl From<InnerCampaign> for Campaign {
    fn from(i: InnerCampaign) -> Self {
        Self {
            id: i.id,
            name: i.name,
            difficulties: i.difficulties.iter().map(|d| *d as u32).collect(),
            created_at: i.created_at.unix_timestamp(),
        }
    }
}
