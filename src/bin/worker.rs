use reqwest::Client;
use rocket::serde::ser::StdError;
use rocket::time::OffsetDateTime;
use tokio::time::sleep;

use std::error::Error;
use std::time::Duration;

use fizzbuzz::{NextTaskTimeResponse, Task, TaskID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let w = Worker {
        client: Client::new(),
    };
    loop {
        let next_task = match w.check_next_task_time().await {
            Ok(t) => t,
            Err(_) => {
                println!("Could not check time for next task - it's possible the task queue is empty");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        if next_task.time > OffsetDateTime::now_utc() {
            println!("Next task time is {} - deferring", next_task.time);
            sleep(Duration::from_secs(1)).await;
            continue;
        }

        let task = match w.pull_task(next_task.id).await {
            Ok(t) => t,
            Err(_) => {
                println!("Could not pull new task - it's possible the task queue is empty");
                sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        println!("Claimed task {}", task.id);
        task.exec().await;

        let _ = w.complete_task(task.id).await?;
    }
}

struct Worker {
    client: Client,
}

impl Worker {
    async fn check_next_task_time(
        &self,
    ) -> Result<NextTaskTimeResponse, Box<dyn Error>> {
        let req = self
            .client
            .get("http://localhost:8000/tasks/next")
            .build()?;

        let resp = self
            .client
            .execute(req)
            .await?
            .json::<NextTaskTimeResponse>()
            .await?;

        Ok(resp)
    }

    async fn pull_task(
        &self,
        id: TaskID,
    ) -> Result<Task, Box<dyn StdError + 'static>> {
        let req = self
            .client
            .post(format!("http://localhost:8000/tasks/pull/{}", id))
            .build()?;

        Ok(self.client.execute(req).await?.json::<Task>().await?)
    }

    async fn complete_task(
        &self,
        id: TaskID,
    ) -> Result<(), Box<dyn StdError + 'static>> {
        let req = self
            .client
            .post(format!("http://localhost:8000/tasks/{}", id).as_str())
            .build()?;

        let _ = self.client.execute(req).await?;
        Ok(())
    }
}
