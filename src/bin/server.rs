#[macro_use]
extern crate rocket;

use rocket::response::status::Created;
use rocket::serde::json::Json;
use rocket_db_pools::{sqlx, Connection, Database};

use fizzbuzz::{
    CreateTaskRequest, CreateTaskResponse, Filters, NextTaskTimeResponse, Task,
    TaskID, TaskState,
};

type Result<T, E = rocket::response::Debug<sqlx::Error>> =
    std::result::Result<T, E>;

#[derive(Database)]
#[database("fizzbuzz-task-db")]
struct TaskDB(sqlx::PgPool);

#[launch]
async fn launch() -> _ {
    rocket::build().attach(TaskDB::init()).mount(
        "/tasks",
        routes![
            create_task,
            list_tasks,
            get_task,
            delete_task,
            get_next_task_time,
            pull_task_by_id,
            complete_task,
        ],
    )
}

#[post("/", format = "application/json", data = "<task>")]
async fn create_task(
    mut db: Connection<TaskDB>,
    task: Json<CreateTaskRequest>,
) -> Result<Created<Json<CreateTaskResponse>>> {
    let req = task.into_inner();
    let record = sqlx::query!(
		"INSERT INTO tasks (tasktype, time, status) VALUES ($1, $2, $3) RETURNING id",
		req.typ.to_string(),
		req.time,
		TaskState::Waiting.to_string(),
	)
    .fetch_one(&mut *db)
    .await?;

    let resp = Json(CreateTaskResponse { id: record.id });
    Ok(Created::new("/tasks").body(resp))
}

#[get("/<id>")]
async fn get_task(
    mut db: Connection<TaskDB>,
    id: TaskID,
) -> Result<Option<Json<Task>>> {
    let r = sqlx::query!("SELECT * from tasks where id = $1", id)
        .fetch_optional(&mut *db)
        .await?;

    match r {
        Some(task) => Ok(Some(Json(Task {
            typ: task.tasktype.clone().try_into().unwrap(),
            status: task.status.clone().try_into().unwrap(),
            time: task.time,
            id: task.id,
        }))),
        None => Ok(None),
    }
}

#[delete("/<id>")]
async fn delete_task(
    mut db: Connection<TaskDB>,
    id: TaskID,
) -> Result<Option<()>> {
    sqlx::query!("DELETE from tasks where id = $1", id)
        .execute(&mut *db)
        .await?;

    Ok(Some(()))
}

#[get("/next")]
async fn get_next_task_time(
    mut db: Connection<TaskDB>,
) -> Result<Option<Json<NextTaskTimeResponse>>> {
    let r = sqlx::query!("SELECT id, time FROM tasks WHERE status = 'Waiting' ORDER BY time, id LIMIT 1 ")
	.fetch_optional(&mut *db)
	.await?;

    match r {
        Some(task) => Ok(Some(Json(NextTaskTimeResponse {
            time: task.time,
            id: task.id,
        }))),
        None => Ok(None),
    }
}

#[post("/pull/<id>")]
async fn pull_task_by_id(
    mut db: Connection<TaskDB>,
    id: TaskID,
) -> Result<Option<Json<Task>>> {
    let r = sqlx::query!(
        r#"
UPDATE tasks
SET status = 'Claimed'
WHERE status = 'Waiting' 
AND id = $1
RETURNING id, tasktype, time, status
		"#,
        id
    )
    .fetch_optional(&mut *db)
    .await?;

    match r {
        Some(task) => Ok(Some(Json(Task {
            typ: task.tasktype.clone().try_into().unwrap(),
            status: task.status.clone().try_into().unwrap(),
            time: task.time,
            id: task.id,
        }))),
        None => Ok(None),
    }
}

#[post("/<id>")]
async fn complete_task(
    mut db: Connection<TaskDB>,
    id: TaskID,
) -> Result<Option<Json<Task>>> {
    let r =
		sqlx::query!("UPDATE tasks SET status = 'Complete' WHERE id = $1 AND status = 'Claimed' RETURNING *", id)
			.fetch_optional(&mut *db)
			.await?;

    match r {
        Some(task) => Ok(Some(Json(Task {
            typ: task.tasktype.clone().try_into().unwrap(),
            status: task.status.clone().try_into().unwrap(),
            time: task.time,
            id: task.id,
        }))),
        None => Ok(None),
    }
}

// This function is more repetitive than I would have liked, particularly when it comes to
// the closure passed into each `map` call. It would be nice to turn that into a function
// that I can call whenever I need to convert a record into a task. Unfortunately, the type
// of `record` is opaque here.
//
// Writing a function that takes a string and passing that to `query!()` would also work.
// Unfortunately, that macro expects a string literal, and not a variable name, for some reason.
//
// Long story short, this code is ugly and repetitive, but it will have to do for now.
#[get("/?<filters>")]
async fn list_tasks(
    mut db: Connection<TaskDB>,
    filters: Filters,
) -> Result<Json<Vec<Task>>> {
    let tasks: Vec<Task> = match (filters.type_filter, filters.status_filter) {
        // No filter
        (None, None) => sqlx::query!("SELECT * FROM tasks")
            .fetch_all(&mut *db)
            .await?
            .iter()
            .map(|record| Task {
                typ: record.tasktype.clone().try_into().unwrap(),
                status: record.status.clone().try_into().unwrap(),
                time: record.time,
                id: record.id,
            })
            .collect(),

        // Type filter
        (Some(t), None) => sqlx::query!(
            "SELECT * FROM tasks WHERE tasktype = $1",
            t.to_string()
        )
        .fetch_all(&mut *db)
        .await?
        .iter()
        .map(|record| Task {
            typ: record.tasktype.clone().try_into().unwrap(),
            status: record.status.clone().try_into().unwrap(),
            time: record.time,
            id: record.id,
        })
        .collect(),

        // Status filter
        (None, Some(s)) => {
            sqlx::query!("SELECT * FROM tasks WHERE status = $1", s.to_string())
                .fetch_all(&mut *db)
                .await?
                .iter()
                .map(|record| Task {
                    typ: record.tasktype.clone().try_into().unwrap(),
                    status: record.status.clone().try_into().unwrap(),
                    time: record.time,
                    id: record.id,
                })
                .collect()
        }

        // Type and status filters
        (Some(t), Some(s)) => sqlx::query!(
            "SELECT * FROM tasks WHERE tasktype = $1 AND status = $2",
            t.to_string(),
            s.to_string(),
        )
        .fetch_all(&mut *db)
        .await?
        .iter()
        .map(|record| Task {
            typ: record.tasktype.clone().try_into().unwrap(),
            status: record.status.clone().try_into().unwrap(),
            time: record.time,
            id: record.id,
        })
        .collect(),
    };

    Ok(Json(tasks))
}
