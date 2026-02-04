// Repository trait definitions for scheduling service
// These define the interface for data access

// Example trait:
// #[async_trait]
// pub trait ScheduleJobRepository {
//     async fn create(&self, job: ScheduleJob) -> Result<ScheduleJob, Error>;
//     async fn find_by_id(&self, id: &str) -> Result<Option<ScheduleJob>, Error>;
//     async fn update_status(&self, id: &str, status: JobStatus) -> Result<(), Error>;
// }
