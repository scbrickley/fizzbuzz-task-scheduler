#[macro_use]
extern crate rocket;

use rocket::serde::json::Json;

use fizzbuzz::{db, CreateTaskRequest, Filters};

#[launch]
fn launch_server() -> _ {
    rocket::build()
        .mount("/", routes![create_task, list_tasks, delete_task, get_task])
}

// Test by starting the server and running:
// curl -X POST -H 'content-type: application/json' -d '{"type": "Fizz", "time": "2023-03-26T11:59:59Z"}' localhost:8000/tasks
#[post("/tasks", format = "application/json", data = "<task>")]
async fn create_task(task: Json<CreateTaskRequest>) -> String {
    match db::create_task(task.into_inner()).await {
        Ok(id) => format!("{}", id),
        Err(e) => format!("{}", e),
    }
}

// Test by starting the server and running:
// curl -X GET 'localhost:8000/tasks?filters.type=fizzbuzz&filters.status=complete'
#[get("/tasks?<filters>")]
async fn list_tasks(filters: Filters) -> String {
    match db::list_tasks(filters).await {
        Ok(s) => s,
        Err(e) => format!("{}", e),
    }
}

// Test by starting the server and running:
// curl -X DELETE localhost:8000/tasks/11
#[delete("/tasks/<id>")]
async fn delete_task(id: i32) -> String {
    match db::delete_task(id).await {
        Ok(s) => s,
        Err(e) => format!("{}", e),
    }
}

// Test by starting the server and running:
// curl -X GET localhost:8000/tasks/12
#[get("/tasks/<id>")]
async fn get_task(id: i32) -> String {
    match db::get_task(id).await {
        Ok(s) => s,
        Err(e) => format!("{}", e),
    }
}
