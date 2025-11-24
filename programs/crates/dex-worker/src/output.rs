use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct WorkerOutput<T> {
    pub ok: bool,
    pub job: &'static str,
    pub data: T,
    pub ts: i64,
}

impl<T: Serialize> WorkerOutput<T> {
    pub fn success(job: &'static str, data: T) -> Self {
        Self {
            ok: true,
            job,
            data,
            ts: chrono::Utc::now().timestamp_millis(),
        }
    }

    pub fn print_json(&self) -> anyhow::Result<()> {
        println!("{}", serde_json::to_string_pretty(self)?);
        Ok(())
    }
}
