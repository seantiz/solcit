use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Job {
   pub id: i32,
   pub uniqueid: String,
   pub title: String,
   pub company: String,
   pub location: String,
   pub salary: String,
   pub jobkey: String,
   pub fetched_date: String,
   pub read: bool,
   pub appliedto: bool,
   pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobUpdate {
    pub read: Option<bool>,
    pub appliedto: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stats {
    pub uniquejobs: i32,
    pub appliedjobs: i32,
}

#[derive(Serialize)]
pub struct ParsedDetails {
    pub experience: String,
    pub interests: String,
    pub projects: String,
    pub education: String,
    pub certificates: String,
}