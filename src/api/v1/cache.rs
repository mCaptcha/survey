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
use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct ChallengeCache {
    store: HashMap<Uuid, Arc<Vec<i32>>>,
    db: PgPool,
}

impl ChallengeCache {
    pub fn new(db: PgPool) -> Self {
        let store = HashMap::default();
        Self { db, store }
    }

    async fn get_db(&self, id: &Uuid) -> Vec<i32> {
        struct Foo {
            challenges: Vec<i32>,
        }
        let res = sqlx::query_as!(
            Foo,
            "SELECT challenges FROM survey_surveys WHERE id = $1",
            &id
        )
        .fetch_one(&self.db)
        .await
        .unwrap();

        res.challenges
    }

    pub async fn get(&mut self, k: &Uuid) -> Arc<Vec<i32>> {
        match self.store.get(k) {
            Some(val) => val.clone(),
            None => {
                let resp = self.get_db(k).await;
                let resp = Arc::new(resp);
                self.store.insert(k.clone(), resp.clone());
                resp
            }
        }
    }
}
