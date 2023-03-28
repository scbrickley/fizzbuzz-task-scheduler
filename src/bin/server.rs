#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;

use fizzbuzz::{CreateTaskRequest, Filters};

#[launch]
fn launch_server() -> _ {
	rocket::build()
		.mount("/", routes![create_task, list_tasks, delete_task, get_task])
}

// Stub for the create-task endpoint. Test by starting the server and running:
// curl -X POST -H 'content-type: application/json' -d '{"type": "Fizz", "time": "2023-03-26T11:59:59Z"}' localhost:8000/tasks
#[post("/tasks", format = "application/json", data = "<task>")]
fn create_task(task: Json<CreateTaskRequest>) -> String {
	format!(
		"Creating task of type {} at timestamp {}",
		task.typ, task.time
	)
}

// Stub for the list-tasks endpoint. Test by starting the server and running:
// curl -X GET 'localhost:8000/tasks?filters.type=fizzbuzz&filters.status=complete'
#[get("/tasks?<filters>")]
fn list_tasks(filters: Filters) -> String {
	format!("{:?}", filters)
}

// Stub for the delete-task endpoint. Test by starting the server and running:
// curl -X DELETE localhost:8000/tasks/11
#[delete("/tasks/<id>")]
fn delete_task(id: i32) -> String {
	format!("deleting task with id {}", id)
}

// Stub for the get-task endpoint. Test by starting the server and running:
// curl -X GET localhost:8000/tasks/12
#[get("/tasks/<id>")]
fn get_task(id: i32) -> String {
	format!("getting task with id {}", id)
}
