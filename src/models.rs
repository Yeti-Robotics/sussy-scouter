use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId, Bson, Document, DateTime},
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::{aggregation, ping_str};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub discord_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleBlock {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub start_time: DateTime,
    pub end_time: DateTime,
    #[serde(default)]
    pub blue1: Option<ObjectId>,
    #[serde(default)]
    pub blue2: Option<ObjectId>,
    #[serde(default)]
    pub blue3: Option<ObjectId>,
    #[serde(default)]
    pub red1: Option<ObjectId>,
    #[serde(default)]
    pub red2: Option<ObjectId>,
    #[serde(default)]
    red3: Option<ObjectId>,
    pub min_30: bool,
    pub min_10: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopulatedScheduleBlock {
    #[serde(rename = "_id")]
    pub _id: ObjectId,
    pub start_time: DateTime,
    pub end_time: DateTime,
    #[serde(default)]
    pub blue1: Option<User>,
    #[serde(default)]
    pub blue2: Option<User>,
    #[serde(default)]
    pub blue3: Option<User>,
    #[serde(default)]
    pub red1: Option<User>,
    #[serde(default)]
    pub red2: Option<User>,
    #[serde(default)]
    pub red3: Option<User>,
    pub min_30: bool,
    pub min_10: bool,
}

impl ScheduleBlock {
    pub async fn find_all(
        collection: &Collection<Self>,
    ) -> Result<Vec<Self>, mongodb::error::Error> {
        collection.find(None, None).await?.try_collect().await
    }

    /// Returns all schedgy blocks with users sorted from most recent
    pub async fn find_all_populated(
        collection: &Collection<Self>,
    ) -> Result<Vec<PopulatedScheduleBlock>, mongodb::error::Error> {
        Ok(collection
            .aggregate(aggregation::POPULATE_BLOCKS_AGGREGATION.clone(), None)
            .await?
            .try_collect::<Vec<Document>>()
            .await?
            .into_iter()
            .map(|doc| {
                serde_json::from_value::<PopulatedScheduleBlock>(
                    <Document as Into<Bson>>::into(doc).into(),
                )
                .expect("Aggregation produced invalid populated schedgy block!!!")
            })
            .collect())
    }
}

impl PopulatedScheduleBlock {
    pub async fn update_min_30(
        &mut self,
        collection: &Collection<ScheduleBlock>,
    ) -> Result<(), mongodb::error::Error> {
        collection
            .update_one(
                doc! { "_id": self._id },
                doc! { "$set": { "min30": true } },
                None,
            )
            .await
            .map(|_| {
                self.min_30 = true;
            })
    }

    pub async fn update_min_10(
        &mut self,
        collection: &Collection<ScheduleBlock>,
    ) -> Result<(), mongodb::error::Error> {
        collection
            .update_one(
                doc! { "_id": self._id },
                doc! { "$set": { "min10": true } },
                None,
            )
            .await
            .map(|_| {
                self.min_10 = true;
            })
    }

    pub fn pings(&self) -> String {
		// 24 chars per pings for 6 people, one allocation
		let mut ping = String::with_capacity(24 * 6);
        if let Some(blue1) = self.blue1.as_ref() {
			ping.push_str(&ping_str(&blue1.discord_id))
		};
		if let Some(blue2) = self.blue2.as_ref() {
			ping.push_str(&ping_str(&blue2.discord_id))
		};
		if let Some(blue3) = self.blue3.as_ref() {
			ping.push_str(&ping_str(&blue3.discord_id))
		};
		if let Some(red1) = self.red1.as_ref() {
			ping.push_str(&ping_str(&red1.discord_id))
		};
		if let Some(red2) = self.red2.as_ref() {
			ping.push_str(&ping_str(&red2.discord_id))
		};
		if let Some(red3) = self.red3.as_ref() {
			ping.push_str(&ping_str(&red3.discord_id))
		};

		ping
    }
}
