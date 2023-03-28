use rocket::serde::json;
use rocket::serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};

use crate::{CreateTaskRequest, Filters, Task, TaskID, TaskState};

#[derive(Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct State {
	pub last_id: TaskID,
	pub tasks: Vec<Task>,
}

pub async fn open_db_file() -> std::io::Result<File> {
	let file = OpenOptions::new()
		.read(true)
		.write(true)
		.create(true)
		.open(db_filepath())
		.await?;

	Ok(file)
}

fn db_filepath() -> PathBuf {
	let mut path = home::home_dir().unwrap();
	path.push(".fizzbuzz-tasks.json");
	path
}

async fn read_state() -> std::io::Result<State> {
	let mut contents = vec![];
	let mut file = open_db_file().await?;
	file.read_to_end(&mut contents).await?;
	file.flush().await?;

	let data = std::str::from_utf8(&contents).unwrap();
	let state: State;
	if data.is_empty() {
		state = State {
			last_id: 0,
			tasks: vec![],
		};
	} else {
		state = json::from_str(data).unwrap()
	}

	Ok(state)
}

async fn write_state(state: &State) -> std::io::Result<()> {
	let state_json = json::to_string(&state).unwrap();
	let mut file = open_db_file().await?;
	file.seek(SeekFrom::Start(0)).await?;
	file.set_len(0).await?;
	file.write_all(state_json.as_bytes()).await?;
	file.flush().await?;
	Ok(())
}

pub async fn create_task(req: CreateTaskRequest) -> std::io::Result<TaskID> {
	let mut state = read_state().await?;
	state.last_id += 1;
	state.tasks.push(Task {
		typ: req.typ,
		time: req.time,
		id: state.last_id,
		status: TaskState::Scheduled,
	});
	write_state(&state).await?;
	Ok(state.last_id)
}

pub async fn list_tasks(filters: Filters) -> std::io::Result<String> {
	let state = read_state().await?;
	let filtered_tasks: Vec<Task> = match filters {
		Filters {
			type_filter: None,
			status_filter: None,
		} => state.tasks,
		_ => state
			.tasks
			.iter()
			.cloned()
			.filter(|task| {
				let tmatch = if let Some(t) = &filters.type_filter {
					*t == task.typ
				} else {
					true
				};

				let smatch = if let Some(s) = &filters.status_filter {
					*s == task.status
				} else {
					true
				};

				tmatch && smatch
			})
			.collect(),
	};
	Ok(json::to_string(&filtered_tasks).unwrap())
}

pub async fn get_task(id: TaskID) -> std::io::Result<String> {
	let state = read_state().await?;
	match state.tasks.iter().find(|t| t.id == id) {
		Some(t) => Ok(json::to_string(&t).unwrap()),
		None => Ok(format!("No task found with id {}", id)),
	}
}

pub async fn delete_task(id: TaskID) -> std::io::Result<String> {
	let mut state = read_state().await?;
	match state.tasks.iter().position(|t| t.id == id) {
		Some(pos) => {
			let removed = state.tasks.remove(pos);
			write_state(&state).await?;
			Ok(json::to_string(&removed).unwrap())
		}
		None => Ok(format!("No task found with id {}", id)),
	}
}

pub async fn pull_pending_task() -> std::io::Result<Option<Task>> {
	let tasks = pending_tasks_by_timestamp().await?;
	match tasks.len() {
		1.. => {
			let next_task = tasks[0].clone();
			Ok(Some(next_task))
		}
		_ => Ok(None),
	}
}

pub async fn pending_tasks_by_timestamp() -> std::io::Result<Vec<Task>> {
	let state = read_state().await?;
	let mut pending_tasks: Vec<Task> = state
		.tasks
		.iter()
		.cloned()
		.filter(|t| t.status == TaskState::Scheduled)
		.collect();
	pending_tasks.sort_by(|a, b| a.time.cmp(&b.time));
	Ok(pending_tasks)
}

pub async fn complete_task(id: TaskID) -> std::io::Result<()> {
	let mut state = read_state().await?;
	match state.tasks.iter().position(|t| t.id == id) {
		Some(pos) => {
			let mut finished = state.tasks[pos].clone();
			finished.status = TaskState::Complete;
			state.tasks[pos] = finished;
			write_state(&state).await?;
			Ok(())
		}
		None => Ok(()),
	}
}
