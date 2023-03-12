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
use std::future::Future;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sqlx::types::time::OffsetDateTime;
use sqlx::types::Uuid;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot::{self, error::TryRecvError, Sender};

use crate::api::v1::admin::campaigns::runners::get_results;
use crate::api::v1::admin::campaigns::SurveyResponse;
use crate::{errors::ServiceResult, AppData, Settings};

const CAMPAIGN_INFO_FILE: &str = "campaign.json";
const BENCHMARK_FILE: &str = "benchmark.csv";

pub struct Archiver {
    base_path: String,
}

pub struct Archive {
    now: i64,
    base_path: String,
    campaign: Uuid,
}

impl Archive {
    pub fn new(campaign: Uuid, base_path: String) -> Self {
        let now = OffsetDateTime::now_utc().unix_timestamp();
        Self {
            now,
            campaign,
            base_path,
        }
    }

    fn campaign_path(&self) -> PathBuf {
        Path::new(&self.base_path).join(&self.campaign.to_string())
    }
    fn archive_path_now(&self) -> PathBuf {
        self.campaign_path().join(self.now.to_string())
    }

    fn campaign_file_path(&self) -> PathBuf {
        self.archive_path_now().join(CAMPAIGN_INFO_FILE)
    }

    fn benchmark_file_path(&self) -> PathBuf {
        self.archive_path_now().join(BENCHMARK_FILE)
    }
}

impl Archiver {
    pub fn new(s: &Settings) -> Self {
        Archiver {
            base_path: s.publish.dir.clone(),
        }
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

    async fn write_campaign_file(&self, c: &Campaign, a: &Archive) -> ServiceResult<()> {
        let archive_path = a.archive_path_now();
        Self::create_dir_util(&archive_path).await?;
        let campaign_file_path = a.campaign_file_path();
        let contents = serde_json::to_string(c).unwrap();
        //        fs::write(campaign_file_path, contents).await.unwrap();
        let mut file = fs::File::create(&campaign_file_path).await.unwrap();
        file.write_all(contents.as_bytes()).await.unwrap();
        file.flush().await.unwrap();

        Ok(())
    }

    fn get_headers(c: &Campaign) -> Vec<String> {
        let mut keys = vec![
            "ID".to_string(),
            "user".to_string(),
            "device_user_provided".to_string(),
            "device_software_recognised".to_string(),
            "threads".to_string(),
            "submitted_at".to_string(),
            "submission_type".to_string(),
        ];

        let mut diff_order = Vec::with_capacity(c.difficulties.len());

        for d in c.difficulties.iter() {
            diff_order.push(d);
            keys.push(format!("Difficulty {}", d));
        }

        keys
    }

    fn extract_record(c: &Campaign, r: SurveyResponse) -> Vec<String> {
        let mut rec = vec![
            r.id.to_string(),
            r.user.id.to_string(),
            r.device_user_provided,
            r.device_software_recognised,
            r.threads.map_or_else(|| "-".into(), |v| v.to_string()),
            r.submitted_at.to_string(),
            r.submission_type.to_string(),
        ];
        for d in c.difficulties.iter() {
            let bench = r
                .benches
                .iter()
                .find(|b| b.difficulty == *d as i32)
                .map_or_else(|| "-".into(), |v| v.duration.to_string());
            rec.push(bench);
        }
        rec
    }

    async fn write_benchmark_file(
        &self,
        c: &Campaign,
        archive: &Archive,
        data: &AppData,
    ) -> ServiceResult<()> {
        let archive_path = archive.archive_path_now();
        Self::create_dir_util(&archive_path).await?;

        let benchmark_file_path = archive.benchmark_file_path();
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
            &Uuid::parse_str(&c.id.to_string()).unwrap()
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

        let keys = Self::get_headers(c);
        wri.write_record(&keys).await.unwrap();

        loop {
            let mut resp = get_results(
                &owner.name,
                &Uuid::parse_str(&c.id.to_string()).unwrap(),
                data,
                page,
                limit,
                None,
            )
            .await?;

            for r in resp.drain(0..) {
                let rec = Self::extract_record(c, r);
                wri.write_record(&rec).await.unwrap();
                wri.flush().await.unwrap();
            }

            if resp.len() < limit {
                break;
            } else {
                page += 1
            }
        }
        Ok(())
    }

    pub async fn init_archive_job(
        self,
        data: AppData,
    ) -> ServiceResult<(Sender<bool>, impl Future)> {
        let (tx, mut rx) = oneshot::channel();

        let job = async move {
            loop {
                //            let rx = self.rx.as_mut().unwrap();
                match rx.try_recv() {
                    // The channel is currently empty
                    Ok(_) => {
                        log::info!("Killing archive loop: received signal");
                        break;
                    }
                    Err(TryRecvError::Empty) => {
                        let _ = self.archive(&data).await;

                        tokio::time::sleep(std::time::Duration::new(
                            data.settings.publish.duration,
                            0,
                        ))
                        .await;
                    }
                    Err(TryRecvError::Closed) => break,
                }

                let _ = self.archive(&data).await;
            }
        };
        let job_fut = tokio::spawn(job);
        Ok((tx, job_fut))
    }

    pub async fn archive(&self, data: &AppData) -> ServiceResult<()> {
        let mut db_campaigns = sqlx::query_as!(
            InnerCampaign,
            "SELECT ID, name, difficulties, created_at FROM survey_campaigns"
        )
        .fetch_all(&data.db)
        .await?;
        for c in db_campaigns.drain(0..) {
            let archive = Archive::new(c.id.clone(), self.base_path.clone());
            let campaign: Campaign = c.into();
            self.write_campaign_file(&campaign, &archive).await?;
            self.write_benchmark_file(&campaign, &archive, data).await?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InnerCampaign {
    id: Uuid,
    name: String,
    difficulties: Vec<i32>,
    created_at: OffsetDateTime,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Campaign {
    pub id: uuid::Uuid,
    pub name: String,
    pub difficulties: Vec<u32>,
    pub created_at: i64,
}

impl From<InnerCampaign> for Campaign {
    fn from(i: InnerCampaign) -> Self {
        Self {
            id: uuid::Uuid::parse_str(&i.id.to_string()).unwrap(),
            name: i.name,
            difficulties: i.difficulties.iter().map(|d| *d as u32).collect(),
            created_at: i.created_at.unix_timestamp(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use csv_async::StringRecord;
    use futures::stream::StreamExt;

    use crate::api::v1::bench::Submission;
    use crate::api::v1::bench::SubmissionType;
    use crate::*;

    use super::*;
    use mktemp::Temp;

    #[test]
    fn archive_path_works() {
        let mut settings = Settings::new().unwrap();
        let tmp_dir = Temp::new_dir().unwrap();
        settings.publish.dir = tmp_dir.join("base_path").to_str().unwrap().into();

        let uuid = Uuid::new_v4();
        let archive = Archive::new(uuid.clone(), settings.publish.dir.clone());
        let archive_path = archive.archive_path_now();
        assert_eq!(
            archive_path,
            Path::new(&settings.publish.dir)
                .join(&uuid.to_string())
                .join(&archive.now.to_string())
        );

        let campaign_file_path = archive.campaign_file_path();
        assert_eq!(
            campaign_file_path,
            Path::new(&settings.publish.dir)
                .join(&uuid.to_string())
                .join(&archive.now.to_string())
                .join(CAMPAIGN_INFO_FILE)
        );

        let benchmark_file_path = archive.benchmark_file_path();
        assert_eq!(
            benchmark_file_path,
            Path::new(&settings.publish.dir)
                .join(&uuid.to_string())
                .join(&archive.now.to_string())
                .join(BENCHMARK_FILE)
        );
    }

    #[actix_rt::test]
    async fn archive_is_correct_test() {
        use crate::tests::*;

        const NAME: &str = "arciscorrecttesuser";
        const EMAIL: &str = "archive_is_correct_testuser@testadminuser.com";
        const PASSWORD: &str = "longpassword2";

        const DEVICE_USER_PROVIDED: &str = "foo";
        const DEVICE_SOFTWARE_RECOGNISED: &str = "Foobar.v2";
        const THREADS: i32 = 4;

        {
            let data = get_test_data().await;
            delete_user(NAME, &data).await;
        }

        //let campaign: Campaign = c.into();
        //let archive = Archive::new(campaign.id.clone(), self.base_path.clone());
        //self.write_campaign_file(&campaign, &archive).await?;
        //self.write_benchmark_file(&campaign, &archive, data).await?;

        let (data, _creds, signin_resp) =
            register_and_signin(NAME, EMAIL, PASSWORD).await;
        let cookies = get_cookie!(signin_resp);
        let survey = get_survey_user(data.clone()).await;
        let survey_cookie = get_cookie!(survey);
        let campaign = create_new_campaign(NAME, data.clone(), cookies.clone()).await;
        let campaign_config =
            get_campaign_config(&campaign, data.clone(), survey_cookie.clone()).await;

        assert_eq!(DIFFICULTIES.to_vec(), campaign_config.difficulties);

        let submit_payload = Submission {
            device_user_provided: DEVICE_USER_PROVIDED.into(),
            device_software_recognised: DEVICE_SOFTWARE_RECOGNISED.into(),
            threads: THREADS,
            benches: BENCHES.clone(),
            submission_type: SubmissionType::Wasm,
        };

        let _proof =
            submit_bench(&submit_payload, &campaign, survey_cookie, data.clone()).await;

        let campaign_id = Uuid::from_str(&campaign.campaign_id).unwrap();
        let db_campaign = sqlx::query_as!(
            InnerCampaign,
            "SELECT ID, name, difficulties, created_at FROM survey_campaigns WHERE ID = $1",
            campaign_id,
        )
        .fetch_one(&data.db)
        .await.unwrap();
        let campaign: Campaign = db_campaign.into();

        let archive = Archive::new(
            Uuid::parse_str(&campaign.id.to_string()).unwrap(),
            data.settings.publish.dir.clone(),
        );
        let archiver = Archiver::new(&data.settings);
        archiver.archive(&AppData::new(data.clone())).await.unwrap();
        let contents: Campaign = serde_json::from_str(
            &fs::read_to_string(&archive.campaign_file_path())
                .await
                .unwrap(),
        )
        .unwrap();
        assert_eq!(contents, campaign);

        let page = 0;
        let limit = 10;
        let mut responses = get_results(
            NAME,
            &campaign_id,
            &AppData::new(data.clone()),
            page,
            limit,
            None,
        )
        .await
        .unwrap();
        assert_eq!(responses.len(), 1);
        let r = responses.pop().unwrap();
        let rec = Archiver::extract_record(&campaign, r);

        let mut rdr = csv_async::AsyncReader::from_reader(
            fs::File::open(archive.benchmark_file_path()).await.unwrap(),
        );

        let mut records = rdr.records();
        assert_eq!(
            records.next().await.unwrap().unwrap(),
            StringRecord::from(rec)
        );
    }
}
