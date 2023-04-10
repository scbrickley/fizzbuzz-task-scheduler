use derive_more::Display;
use rocket::form::{FromForm, FromFormField};
use rocket::serde::{Deserialize, Serialize};
use rocket::time::OffsetDateTime;

use std::time::Duration;

pub type TaskID = i32;

#[derive(Debug, Display)]
#[display(fmt = "{}", msg)]
pub struct SchedulerError {
    msg: String,
}

impl std::error::Error for SchedulerError {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Task {
    pub typ: TaskType,
    pub status: TaskState,
    pub id: TaskID,
    #[serde(with = "rocket::time::serde::rfc3339")]
    pub time: OffsetDateTime,
}

impl TryFrom<std::string::String> for TaskType {
    type Error = SchedulerError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "Fizz" => Ok(TaskType::Fizz),
            "Buzz" => Ok(TaskType::Buzz),
            "FizzBuzz" => Ok(TaskType::FizzBuzz),
            _ => Err(SchedulerError {
                msg: "Invalid task type".to_string(),
            }),
        }
    }
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
    #[display(fmt = "Waiting")]
    Waiting,
    #[display(fmt = "Claimed")]
    Claimed,
    #[display(fmt = "Complete")]
    Complete,
}

impl TryFrom<std::string::String> for TaskState {
    type Error = SchedulerError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "Waiting" => Ok(TaskState::Waiting),
            "Claimed" => Ok(TaskState::Claimed),
            "Complete" => Ok(TaskState::Complete),
            _ => Err(SchedulerError {
                msg: "Invalid task state".to_string(),
            }),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTaskRequest {
    #[serde(rename = "type")]
    pub typ: TaskType,
    #[serde(with = "rocket::time::serde::rfc3339")]
    pub time: OffsetDateTime,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct CreateTaskResponse {
    pub id: TaskID,
}

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct NextTaskTimeResponse {
    #[serde(with = "rocket::time::serde::rfc3339")]
    pub time: OffsetDateTime,
    pub id: TaskID,
}

#[derive(Debug, Deserialize, Serialize, FromForm)]
#[serde(crate = "rocket::serde")]
pub struct Filters {
    #[field(name = "type")]
    pub type_filter: Option<TaskType>,
    #[field(name = "status")]
    pub status_filter: Option<TaskState>,
}
