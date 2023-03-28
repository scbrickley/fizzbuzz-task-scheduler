pub mod db;

use chrono::{DateTime, Utc};
use derive_more::Display;
use rocket::form::{FromForm, FromFormField};
use rocket::serde::{Deserialize, Serialize};

use std::time::Duration;

pub type TaskID = i32;

#[derive(Clone, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Task {
    pub typ: TaskType,
    pub status: TaskState,
    pub id: TaskID,
    pub time: DateTime<Utc>,
}

impl Task {
    pub async fn exec(&self) {
        let nap_time = match self.typ {
            TaskType::Fizz => 3,
            TaskType::Buzz => 5,
            TaskType::FizzBuzz => 0,
        };

        match self.typ {
            TaskType::Fizz | TaskType::Buzz => {
                println!(
                    "Executing {} task - sleeping for {} seconds",
                    self.typ, nap_time
                );
                tokio::time::sleep(Duration::from_secs(nap_time)).await;
                println!("{}: {}", self.typ, self.id)
            }
            TaskType::FizzBuzz => {
                println!("{}: {}", self.typ, self.time)
            }
        }
    }
}

#[derive(
    Clone, Debug, Deserialize, Display, Serialize, FromFormField, PartialEq,
)]
#[serde(crate = "rocket::serde")]
pub enum TaskType {
    #[display(fmt = "Fizz")]
    Fizz,
    #[display(fmt = "Buzz")]
    Buzz,
    #[display(fmt = "FizzBuzz")]
    FizzBuzz,
}

#[derive(
    Clone, Debug, Deserialize, Display, Serialize, FromFormField, PartialEq,
)]
#[serde(crate = "rocket::serde")]
pub enum TaskState {
    #[display(fmt = "Scheduled")]
    Scheduled,
    #[display(fmt = "Complete")]
    Complete,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTaskRequest {
    #[serde(rename = "type")]
    pub typ: TaskType,
    pub time: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Filters {
    #[field(name = "type")]
    type_filter: Option<TaskType>,
    #[field(name = "status")]
    status_filter: Option<TaskState>,
}
