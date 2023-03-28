use chrono::{DateTime, Utc};
use derive_more::Display;
use rocket::form::{FromForm, FromFormField};
use rocket::serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Task {
	pub typ: TaskType,
	pub state: TaskState,
	pub id: i32,
}

#[derive(Debug, Deserialize, Display, Serialize, FromFormField)]
#[serde(crate = "rocket::serde")]
pub enum TaskType {
	#[display(fmt = "Fizz")]
	Fizz,
	#[display(fmt = "Buzz")]
	Buzz,
	#[display(fmt = "FizzBuzz")]
	FizzBuzz,
}

#[derive(Debug, Deserialize, Display, Serialize, FromFormField)]
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
