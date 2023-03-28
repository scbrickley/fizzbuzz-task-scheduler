use fizzbuzz::db;

use chrono::Utc;

#[tokio::main]
async fn main() -> std::io::Result<()> {
	loop {
		let now = Utc::now();
		let next_task = db::pull_pending_task().await?;
		if let Some(t) = next_task {
			if t.time <= now {
				t.exec().await;
				db::complete_task(t.id).await?;
			}
		}
	}
}
