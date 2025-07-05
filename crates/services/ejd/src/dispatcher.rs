//! Core dispatcher logic for the EJ Dispatcher Service.
//!
//! The dispatcher manages job queues, builder connections, and coordinates
//! the execution of jobs across available builders. It handles:
//!
//! - Job queuing and distribution
//! - Builder connection management
//! - Job timeout and cancellation
//! - Result collection and persistence
//!
//! The dispatcher runs as a background task that processes events and
//! manages the lifecycle of jobs from submission to completion.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use crate::prelude::*;
use ej_dispatcher_sdk::ejjob::{
    EjBuildResult, EjDeployableJob, EjJob, EjJobCancelReason, EjJobType, EjJobUpdate, EjRunResult,
};
use ej_dispatcher_sdk::ejws_message::EjWsServerMessage;
use ej_models::db::connection::DbConnection;
use ej_models::job::ejjob::EjJobDb;
use ej_models::job::ejjob_logs::EjJobLog;
use ej_models::job::ejjob_results::EjJobResultDb;
use ej_models::job::ejjob_status::EjJobStatus;
use ej_web::ejconfig::board_config_db_to_board_config_api;
use ej_web::ejconnected_builder::EjConnectedBuilder;
use ej_web::ejjob::create_job;
use ej_web::traits::job_result::EjJobResult;
use tokio::time::sleep;
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    task::JoinHandle,
};
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

/// Events that can be sent to the dispatcher.
#[derive(Debug)]
pub enum DispatcherEvent {
    DispatchJob {
        job: EjDeployableJob,
        job_update_tx: Sender<EjJobUpdate>,
        timeout: Duration,
    },
    JobCompleted {
        job_id: Uuid,
        builder_id: Uuid,
    },

    Timeout {
        job_id: Uuid,
    },
}

#[derive(Clone)]
pub struct Dispatcher {
    pub builders: Arc<Mutex<Vec<EjConnectedBuilder>>>,
    pub connection: DbConnection,
    pub tx: Sender<DispatcherEvent>,
}

#[derive(Debug)]
struct DispatchedJob {
    data: EjDeployableJob,
    tx: Sender<EjJobUpdate>,
    timeout: Duration,
}

#[derive(Debug)]
struct RunningJob {
    data: EjDeployableJob,
    job_update_tx: Sender<EjJobUpdate>,
    deployed_builders: HashSet<Uuid>,

    dispatcher_tx: Sender<DispatcherEvent>,
    timeout: Duration,
    timeout_handle: JoinHandle<()>,
}

impl DispatchedJob {
    /// Creates a new DispatchedJob with job data, update channel, and timeout.
    ///
    /// # Arguments
    /// * `data` - The deployable job configuration
    /// * `tx` - Channel for sending job progress updates
    /// * `timeout` - Maximum duration to wait for job completion
    ///
    /// # Returns
    /// A new DispatchedJob instance ready to be started
    pub fn new(data: EjDeployableJob, tx: Sender<EjJobUpdate>, timeout: Duration) -> Self {
        Self { data, tx, timeout }
    }
    /// Starts the job execution by creating a RunningJob with timeout management.
    ///
    /// # Arguments
    /// * `dispatcher_tx` - Channel for sending events back to the dispatcher
    /// * `deployed_builders` - Set of builder IDs that will execute this job
    ///
    /// # Returns
    /// A RunningJob instance with active timeout management
    pub fn start(
        self,
        dispatcher_tx: Sender<DispatcherEvent>,
        deployed_builders: HashSet<Uuid>,
    ) -> RunningJob {
        RunningJob::new(self, dispatcher_tx, deployed_builders)
    }
}
impl RunningJob {
    /// Creates a new RunningJob from a DispatchedJob with timeout management.
    ///
    /// This function sets up the timeout task that will send a timeout event
    /// if the job doesn't complete within the specified duration.
    ///
    /// # Arguments
    /// * `job` - The dispatched job to start running
    /// * `dispatcher_tx` - Channel for sending timeout events
    /// * `deployed_builders` - Set of builders assigned to this job
    ///
    /// # Returns
    /// A RunningJob with active timeout monitoring
    fn new(
        job: DispatchedJob,
        dispatcher_tx: Sender<DispatcherEvent>,
        deployed_builders: HashSet<Uuid>,
    ) -> Self {
        let timeout = job.timeout;
        let tx = dispatcher_tx.clone();
        let job_id = job.data.id;

        Self {
            data: job.data,
            job_update_tx: job.tx,
            timeout: job.timeout,
            deployed_builders,
            timeout_handle: RunningJob::create_task(tx, job_id, timeout),
            dispatcher_tx,
        }
    }
    /// Creates a background task that sends a timeout event after the specified duration.
    ///
    /// # Arguments
    /// * `tx` - Channel for sending the timeout event
    /// * `job_id` - ID of the job to timeout
    /// * `timeout` - Duration to wait before sending timeout
    ///
    /// # Returns
    /// A JoinHandle for the timeout task that can be cancelled
    fn create_task(tx: Sender<DispatcherEvent>, job_id: Uuid, timeout: Duration) -> JoinHandle<()> {
        tokio::spawn(async move {
            sleep(timeout).await;
            if let Err(err) = tx.send(DispatcherEvent::Timeout { job_id }).await {
                error!("Failed to send Timeout Dispatcher Event for job {job_id} - {err}");
            }
        })
    }

    /// Renews the timeout for this job by cancelling the old timeout and creating a new one.
    ///
    /// This is useful when job progress is detected and the timeout should be extended.
    /// The timeout duration remains the same as originally configured.
    fn renew_timeout(&mut self) {
        self.timeout_handle.abort();
        let timeout = self.timeout;
        let tx = self.dispatcher_tx.clone();
        let job_id = self.data.id.clone();
        self.timeout_handle = RunningJob::create_task(tx, job_id, timeout);
    }
}

struct DispatcherPrivate {
    dispatcher: Dispatcher,
    state: DispatcherState,
    pending_jobs: VecDeque<DispatchedJob>,
}

#[derive(Debug)]
enum DispatcherState {
    Idle,
    DispatchedJob { job: RunningJob },
}

impl DispatcherPrivate {
    /// Creates a new dispatcher instance and starts its background processing task.
    ///
    /// # Arguments
    /// * `connection` - Database connection for job and builder management
    ///
    /// # Returns
    /// A tuple containing the dispatcher interface and its background task handle
    fn create(connection: DbConnection) -> (Dispatcher, JoinHandle<()>) {
        let (tx, rx) = channel(32);
        let dispatcher = Dispatcher::new(connection, tx);

        let private = Self {
            dispatcher: dispatcher.clone(),
            state: DispatcherState::Idle,
            pending_jobs: VecDeque::new(),
        };
        let handle = private.start_thread(rx);
        (dispatcher, handle)
    }

    /// Starts the background thread that processes dispatcher events.
    ///
    /// This function runs the main event loop that handles:
    /// - Job dispatch requests
    /// - Job completion notifications
    /// - Job timeout events
    ///
    /// # Arguments
    /// * `rx` - Receiver for dispatcher events
    ///
    /// # Returns
    /// A JoinHandle for the background task
    fn start_thread(mut self, mut rx: Receiver<DispatcherEvent>) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                info!(
                    "New Dispatcher Message. Message {:?}. State {:?}",
                    message, self.state
                );
                let result = match message {
                    DispatcherEvent::DispatchJob {
                        job,
                        job_update_tx,
                        timeout,
                    } => {
                        self.handle_dispatch_job(DispatchedJob::new(job, job_update_tx, timeout))
                            .await
                    }
                    DispatcherEvent::JobCompleted { job_id, builder_id } => {
                        self.handle_job_completed(job_id, builder_id).await
                    }
                    DispatcherEvent::Timeout { job_id } => self.handle_job_timeout(job_id).await,
                };
                if let Err(err) = result {
                    error!("Error while handling last dispatcher message - {}", err);
                }
            }
        })
    }
    /// Dispatches a job to a single builder via WebSocket.
    ///
    /// # Arguments
    /// * `job` - The job to dispatch
    /// * `builder` - The connected builder to send the job to
    ///
    /// # Returns
    /// `true` if the job was successfully sent, `false` if there was an error
    async fn dispatch_job_to_single_builder(
        job: EjDeployableJob,
        builder: &EjConnectedBuilder,
    ) -> bool {
        let message = if job.job_type == EjJobType::BuildAndRun {
            EjWsServerMessage::BuildAndRun(job)
        } else {
            EjWsServerMessage::Build(job)
        };
        if let Err(err) = builder.tx.send(message).await {
            error!("Failed to dispatch builder {:?} - {err}", builder);
            return false;
        }
        trace!("Builder dispatched {:?}", builder);
        return true;
    }
    /// Dispatches a job to all available builders and transitions to running state.
    ///
    /// This function:
    /// - Updates job status to running in the database
    /// - Sends the job to all connected builders
    /// - Tracks which builders successfully received the job
    /// - Transitions to DispatchedJob state or cancels if no builders available
    ///
    /// # Arguments
    /// * `job` - The job to dispatch to builders
    async fn dispatch_job(&mut self, mut job: DispatchedJob) {
        let jobdb = EjJobDb::fetch_by_id(&job.data.id, &self.dispatcher.connection).unwrap();
        if let Err(err) = jobdb.update_status(EjJobStatus::running(), &self.dispatcher.connection) {
            error!(
                "Failed to update job {} status in database {err}",
                job.data.id
            );
        }

        let builders = self.dispatcher.builders.lock().await;
        info!(
            "Dispatching job {} to {} builders",
            job.data.id,
            builders.len()
        );

        let mut dispatched_builders = HashSet::new();
        for builder in builders.iter() {
            if DispatcherPrivate::dispatch_job_to_single_builder(job.data.clone(), &builder).await {
                dispatched_builders.insert(builder.builder.id);
            }
        }
        if dispatched_builders.is_empty() {
            error!("No builder available for job dispatch");
            DispatcherPrivate::send_job_update(
                &mut job.tx,
                EjJobUpdate::JobCancelled(EjJobCancelReason::NoBuilders),
            )
            .await;
            let jobdb = EjJobDb::fetch_by_id(&job.data.id, &self.dispatcher.connection).unwrap();
            if let Err(err) =
                jobdb.update_status(EjJobStatus::running(), &self.dispatcher.connection)
            {
                error!(
                    "Failed to update job {} status in database {err}",
                    job.data.id
                );
            }
        } else {
            DispatcherPrivate::send_job_update(
                &mut job.tx,
                EjJobUpdate::JobStarted {
                    nb_builders: dispatched_builders.len(),
                },
            )
            .await;
            self.state = DispatcherState::DispatchedJob {
                job: job.start(self.dispatcher.tx.clone(), dispatched_builders),
            };
        }
    }
    /// Handles incoming job dispatch requests by either starting the job or queuing it.
    ///
    /// If the dispatcher is idle, the job starts immediately.
    /// If another job is running, the new job is added to the pending queue.
    ///
    /// # Arguments
    /// * `job` - The job to dispatch
    ///
    /// # Returns
    /// Result indicating success or failure
    async fn handle_dispatch_job(&mut self, mut job: DispatchedJob) -> Result<()> {
        match self.state {
            DispatcherState::Idle => self.dispatch_job(job).await,
            DispatcherState::DispatchedJob { .. } => {
                info!(
                    "Can't dispatch new job as there is already one in progress. Adding new job {} to job queue",
                    job.data.id
                );
                DispatcherPrivate::send_job_update(
                    &mut job.tx,
                    EjJobUpdate::JobAddedToQueue {
                        queue_position: self.pending_jobs.len(),
                    },
                )
                .await;
                self.pending_jobs.push_back(job);
            }
        }
        Ok(())
    }
    /// Sends a job update to the update channel, logging any errors.
    ///
    /// # Arguments
    /// * `tx` - The channel to send the update through
    /// * `update` - The job update to send
    async fn send_job_update(tx: &Sender<EjJobUpdate>, update: EjJobUpdate) {
        if let Err(err) = tx.send(update).await {
            error!("Failed to send job update through internal channel {err}");
        }
    }

    /// Handles job completion by collecting results and sending final updates.
    ///
    /// This function:
    /// - Fetches job logs from the database
    /// - Builds result objects with board configuration data
    /// - Sends appropriate completion updates (BuildFinished or RunFinished)
    /// - Handles different job types (Build vs BuildAndRun)
    ///
    /// # Arguments
    /// * `job` - The completed running job
    /// * `connection` - Database connection for fetching results
    ///
    /// # Returns
    /// Result indicating success or failure of the completion handling
    async fn on_job_completed(job: &RunningJob, connection: &DbConnection) -> Result<()> {
        info!("Job {} of type {} complete", job.data.id, job.data.job_type);
        let jobdb = EjJobDb::fetch_by_id(&job.data.id, &connection)?;
        let logsdb = EjJobLog::fetch_with_board_config_by_job_id(&jobdb.id, &connection)?;
        let mut logs = Vec::new();
        for (logdb, board_config_db) in logsdb {
            let config_api = board_config_db_to_board_config_api(board_config_db, connection)?;
            logs.push((config_api, logdb.log));
        }

        if job.data.job_type == EjJobType::Build {
            DispatcherPrivate::send_job_update(
                &job.job_update_tx,
                EjJobUpdate::BuildFinished(EjBuildResult {
                    success: jobdb.success(),
                    logs,
                }),
            )
            .await;
        } else {
            let resultsdb =
                EjJobResultDb::fetch_with_board_config_by_job_id(&jobdb.id, &connection)?;
            let mut results = Vec::new();
            for (resultdb, board_config_db) in resultsdb {
                let config_api = board_config_db_to_board_config_api(board_config_db, connection)?;
                results.push((config_api, resultdb.result));
            }

            DispatcherPrivate::send_job_update(
                &job.job_update_tx,
                EjJobUpdate::RunFinished(EjRunResult {
                    logs,
                    success: jobdb.success(),
                    results,
                }),
            )
            .await;
        }
        Ok(())
    }
    /// Handles the completion of a job by a specific builder.
    ///
    /// This function manages the state transitions when builders complete jobs:
    /// - If in idle state, logs that the builder finished a stale job
    /// - If actively running a job, removes the builder from the deployed set
    /// - When all builders complete, sends final results and processes next job
    /// - Handles cases where builders complete unexpected jobs
    ///
    /// # Arguments
    /// * `completed_job_id` - The ID of the job that was completed
    /// * `builder_id` - The ID of the builder that completed the job
    ///
    /// # Returns
    /// Result indicating success or failure of handling the completion
    async fn handle_job_completed(
        &mut self,
        completed_job_id: Uuid,
        builder_id: Uuid,
    ) -> Result<()> {
        match self.state {
            /* Got a result from a builder that had probably timed out in the past. */
            DispatcherState::Idle => {
                info!(
                    "Builder {} finished job {} but we're currently in idle state",
                    builder_id, completed_job_id
                );
            }
            DispatcherState::DispatchedJob { ref mut job } => {
                info!(
                    "Builder {} finished job {}. Currently deployed builders: {:?}",
                    builder_id, job.data.id, job.deployed_builders
                );
                if job.data.id == completed_job_id {
                    if !job.deployed_builders.remove(&builder_id) {
                        warn!(
                            "Received unexpected JobCompleted message from builder {}",
                            builder_id
                        );
                    }
                    if job.deployed_builders.is_empty() {
                        info!(
                            "Job completed by all builders. # of pending jobs {}",
                            self.pending_jobs.len()
                        );

                        if let Err(err) =
                            DispatcherPrivate::on_job_completed(&job, &self.dispatcher.connection)
                                .await
                        {
                            error!("Failed to send job update {err}");
                        }
                        match self.pending_jobs.pop_front() {
                            Some(new_job) => {
                                self.dispatch_job(new_job).await;
                            }
                            None => {
                                self.state = DispatcherState::Idle;
                            }
                        }
                    }
                } else {
                    info!(
                        "Builder {} finished job {} but we're running job {}",
                        builder_id, completed_job_id, job.data.id
                    );
                    if job.deployed_builders.contains(&builder_id) {
                        info!(
                            "Builder {} has already been dispatched for current job {}",
                            builder_id, job.data.id
                        );
                    } else {
                        info!(
                            "Builder {} has NOT been dispatched for current job {}. Dispatching him",
                            builder_id, job.data.id
                        );
                        let connected_builders = self.dispatcher.builders.lock().await;
                        match connected_builders
                            .iter()
                            .find(|b| b.builder.id == builder_id)
                        {
                            Some(builder) => {
                                info!(
                                    "Dispatching job {} to builder {}",
                                    job.data.id, builder.builder.id
                                );
                                if DispatcherPrivate::dispatch_job_to_single_builder(
                                    job.data.clone(),
                                    &builder,
                                )
                                .await
                                {
                                    job.deployed_builders.insert(builder.builder.id);
                                    job.renew_timeout();
                                }
                            }
                            None => error!(
                                "Couldn't find builder {} that just completed job in the connected builder's list {:?}",
                                builder_id, connected_builders
                            ),
                        }
                    }
                }
            }
        }
        Ok(())
    }
    /// Cancels a running job across all deployed builders.
    ///
    /// This function:
    /// - Sends cancel messages to all builders running the job
    /// - Updates the job status in the database
    /// - Sends cancellation updates to subscribed clients
    /// - Handles communication errors gracefully
    ///
    /// # Arguments
    /// * `builders` - Shared reference to connected builders
    /// * `job` - The running job to cancel
    /// * `connection` - Database connection for status updates
    /// * `reason` - The reason for cancellation (timeout, user request, etc.)
    ///
    /// # Returns
    /// Result indicating success or failure of the cancellation
    async fn cancel_running_job(
        builders: &Arc<Mutex<Vec<EjConnectedBuilder>>>,
        job: &mut RunningJob,
        connection: &DbConnection,
        reason: EjJobCancelReason,
    ) -> Result<()> {
        let connected_builders = builders.lock().await;
        for connected_builder in connected_builders.iter() {
            if !job
                .deployed_builders
                .contains(&connected_builder.builder.id)
            {
                continue;
            }
            if let Err(err) = connected_builder
                .tx
                .send(EjWsServerMessage::Cancel(reason, job.data.id.clone()))
                .await
            {
                error!(
                    "Failed to send cancel message to builder {} - {err}",
                    connected_builder.builder.id
                );
            }
        }
        DispatcherPrivate::cancel_job(&job.data.id, &mut job.job_update_tx, connection, reason)
            .await
    }
    /// Cancels a job by updating its status and notifying clients.
    ///
    /// This function:
    /// - Sends a cancellation update to the job's update channel
    /// - Updates the job status to cancelled in the database
    /// - Logs any database update errors
    ///
    /// # Arguments
    /// * `job_id` - The ID of the job to cancel
    /// * `tx` - The update channel for the job
    /// * `connection` - Database connection for status updates
    /// * `reason` - The reason for cancellation
    ///
    /// # Returns
    /// Result indicating success or failure of the cancellation
    async fn cancel_job(
        job_id: &Uuid,
        tx: &mut Sender<EjJobUpdate>,
        connection: &DbConnection,
        reason: EjJobCancelReason,
    ) -> Result<()> {
        DispatcherPrivate::send_job_update(tx, EjJobUpdate::JobCancelled(reason)).await;
        let jobdb = EjJobDb::fetch_by_id(&job_id, &connection).unwrap();
        if let Err(err) = jobdb.update_status(EjJobStatus::cancelled(), &connection) {
            error!("Failed to update job {} status in database {err}", job_id);
        }
        Ok(())
    }

    /// Handles job timeout by cancelling the job if it's currently running.
    ///
    /// This function:
    /// - Checks if the dispatcher is currently running the timed-out job
    /// - If idle, ignores the timeout as the job was already completed/cancelled
    /// - If running a different job, ignores the timeout for the old job
    /// - If running the matching job, cancels it with a timeout reason
    ///
    /// # Arguments
    /// * `job_id` - The ID of the job that timed out
    ///
    /// # Returns
    /// Result indicating success or failure of handling the timeout
    async fn handle_job_timeout(&mut self, job_id: Uuid) -> Result<()> {
        match self.state {
            DispatcherState::Idle => {
                debug!("Received job timeout but we're already in idle");
                Ok(())
            }
            DispatcherState::DispatchedJob { ref mut job } => {
                if job.data.id != job_id {
                    debug!("Job {} timed out but we're running {}", job_id, job.data.id);
                    return Ok(());
                }

                info!("Job {job_id} timed out. Cancelling it");
                DispatcherPrivate::cancel_running_job(
                    &self.dispatcher.builders,
                    job,
                    &self.dispatcher.connection,
                    EjJobCancelReason::Timeout,
                )
                .await
            }
        }
    }
}
impl Dispatcher {
    /// Creates a new Dispatcher instance with database connection and event channel.
    ///
    /// # Arguments
    /// * `connection` - Database connection for job and builder management
    /// * `tx` - Event channel for sending dispatcher events
    ///
    /// # Returns
    /// A new Dispatcher instance
    fn new(connection: DbConnection, tx: Sender<DispatcherEvent>) -> Self {
        Self {
            connection,
            builders: Arc::new(Mutex::new(Vec::new())),
            tx,
        }
    }
    /// Creates a new Dispatcher and spawns its background task.
    ///
    /// This function creates both the public dispatcher interface and its
    /// private background task that handles job scheduling and builder management.
    ///
    /// # Arguments
    /// * `connection` - Database connection for job and builder management
    ///
    /// # Returns
    /// A tuple containing:
    /// - The public Dispatcher interface
    /// - A JoinHandle for the background dispatcher task
    ///
    /// # Example
    /// ```rust
    /// let (dispatcher, task_handle) = Dispatcher::create(db_connection);
    /// // Use dispatcher for job management
    /// // task_handle will run the background processing
    /// ```
    pub fn create(connection: DbConnection) -> (Self, JoinHandle<()>) {
        DispatcherPrivate::create(connection)
    }

    /// Dispatches a job for execution by available builders.
    ///
    /// This function:
    /// - Validates that builders are available
    /// - Creates a deployable job record in the database
    /// - Sends the job to the dispatcher's background task for execution
    /// - Returns immediately with the deployable job details
    ///
    /// # Arguments
    /// * `job` - The job configuration to execute
    /// * `job_update_tx` - Channel for receiving job progress updates
    /// * `timeout` - Maximum duration to wait for job completion
    ///
    /// # Returns
    /// Result containing the deployable job information, or an error if:
    /// - No builders are available
    /// - Database errors occur
    /// - Communication with background task fails
    ///
    /// # Example
    /// ```rust
    /// let (update_tx, update_rx) = mpsc::channel(100);
    /// let timeout = Duration::from_secs(300);
    ///
    /// let deployable_job = dispatcher.dispatch_job(
    ///     job_config,
    ///     update_tx,
    ///     timeout
    /// ).await?;
    ///
    /// // Listen for updates on update_rx
    /// ```
    pub async fn dispatch_job(
        &mut self,
        job: EjJob,
        job_update_tx: Sender<EjJobUpdate>,
        timeout: Duration,
    ) -> Result<EjDeployableJob> {
        if self.builders.lock().await.len() == 0 {
            return Err(Error::NoBuildersAvailable);
        }
        let job = create_job(job, &mut self.connection)?;

        self.tx
            .send(DispatcherEvent::DispatchJob {
                job: job.clone(),
                job_update_tx,
                timeout,
            })
            .await?;
        Ok(job)
    }

    /// Handles job result submission from builders.
    ///
    /// This function:
    /// - Saves the job result to the database
    /// - Notifies the dispatcher's background task of job completion
    /// - Triggers result processing and potential next job dispatch
    ///
    /// # Arguments
    /// * `result` - The job result from a builder (build or run result)
    ///
    /// # Returns
    /// Result indicating success or failure of result processing
    ///
    /// # Example
    /// ```rust
    /// // When a builder completes a job
    /// let build_result = EjBuildJobResult {
    ///     job_id: job.id,
    ///     builder_id: builder.id,
    ///     success: true,
    ///     // ... other fields
    /// };
    ///
    /// dispatcher.on_job_result(build_result).await?;
    /// ```
    pub async fn on_job_result(&mut self, result: impl EjJobResult) -> Result<()> {
        let job_id = result.job_id();
        let builder_id = result.builder_id();
        result.save(&mut self.connection)?;

        self.tx
            .send(DispatcherEvent::JobCompleted {
                job_id: job_id,
                builder_id: builder_id,
            })
            .await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use diesel::prelude::*;
    use diesel::r2d2::{ConnectionManager, Pool};
    use ej_dispatcher_sdk::ejjob::results::{EjBuilderBuildResult, EjBuilderRunResult};
    use ej_models::db::config::DbConfig;
    use ej_models::db::connection::DbConnection;
    use ej_web::ctx::ctx_client::CtxClient;
    use ej_web::ejconnected_builder::EjConnectedBuilder;
    use std::collections::HashMap;
    use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time::timeout;
    use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
    use uuid::Uuid;

    static INIT: std::sync::Once = std::sync::Once::new();

    fn setup_test_environment() {
        INIT.call_once(|| {
            tracing_subscriber::registry()
                .with(tracing_subscriber::fmt::layer())
                .init();
        });
    }

    struct DbTestContext {
        pub connection: DbConnection,
        base_url: String,
        db_name: String,
    }
    impl DbTestContext {
        pub fn create() -> Self {
            let base_url =
                std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL env variable missing");

            let test_db_name = format!("ej_test_{}", uuid::Uuid::new_v4().simple());

            // Connect to base database to create test database
            let base_manager =
                ConnectionManager::<PgConnection>::new(&format!("{}/postgres", base_url));
            let base_pool = Pool::builder()
                .max_size(1)
                .build(base_manager)
                .expect("Failed to connect to base database");

            {
                let mut conn = base_pool.get().expect("Failed to get connection");
                diesel::sql_query(&format!("CREATE DATABASE {}", test_db_name))
                    .execute(&mut conn)
                    .expect("Failed to create test database");
            }
            let config = DbConfig {
                database_url: format!("{}/{}", base_url, test_db_name),
            };
            let test_connection = DbConnection::new(&config).setup();
            let db_context = DbTestContext {
                connection: test_connection,
                base_url,
                db_name: test_db_name,
            };

            db_context
        }
    }
    impl Drop for DbTestContext {
        fn drop(&mut self) {
            let base_manager =
                ConnectionManager::<PgConnection>::new(&format!("{}/postgres", self.base_url));
            let base_pool = Pool::builder()
                .max_size(1)
                .build(base_manager)
                .expect("Failed to connect to base database for cleanup");

            {
                let mut conn = base_pool
                    .get()
                    .expect("Failed to get connection for cleanup");

                diesel::sql_query(&format!(
                    "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
                    self.db_name
                ))
                .execute(&mut conn)
                .ok();

                diesel::sql_query(&format!("DROP DATABASE IF EXISTS {}", self.db_name))
                    .execute(&mut conn)
                    .ok();
            }
        }
    }

    fn create_builder(builder_id: Uuid, tx: Sender<EjWsServerMessage>) -> EjConnectedBuilder {
        EjConnectedBuilder {
            builder: CtxClient { id: builder_id },
            tx,
            addr: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(10, 0, 0, 2), 11111)),
        }
    }

    fn create_test_job() -> EjJob {
        EjJob {
            job_type: EjJobType::Build,
            commit_hash: String::from("HASH"),
            remote_url: String::from("URL"),
            remote_token: None,
        }
    }

    async fn setup_dispatcher(connection: DbConnection) -> (Dispatcher, JoinHandle<()>) {
        Dispatcher::create(connection)
    }

    macro_rules! test {
        ($test_fn:expr) => {{
            setup_test_environment();
            let result = {
                let context = DbTestContext::create();
                let (dispatcher, handle) = setup_dispatcher(context.connection.clone()).await;
                $test_fn(dispatcher, handle).await
            };
            result
        }};
    }

    #[tokio::test]
    async fn test_dispatch_job_no_builders_available() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let (job_update_tx, _job_update_rx) = mpsc::channel(32);

            let job = create_test_job();
            let result = dispatcher
                .dispatch_job(job, job_update_tx, Duration::from_secs(60))
                .await;
            assert!(result.is_err());
            match result {
                Err(Error::NoBuildersAvailable) => {}
                _ => panic!("Expected NoBuildersAvailable error"),
            }
        });
    }

    #[tokio::test]
    async fn test_dispatch_job_with_single_builder() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let (job_update_tx, mut job_update_rx) = mpsc::channel(32);

            // Add a mock builder
            let builder_id = Uuid::new_v4();
            let (builder_tx, mut builder_rx) = channel(32);
            let builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(builder);

            let job = create_test_job();

            // Dispatch the job
            let result = dispatcher
                .dispatch_job(job, job_update_tx, Duration::from_secs(60))
                .await;
            assert!(result.is_ok());

            let builder_dispatch = timeout(Duration::from_millis(100), builder_rx.recv())
                .await
                .expect("Should receive dispatch")
                .unwrap();
            assert_eq!(builder_dispatch, EjWsServerMessage::Build(result.unwrap()));

            // Should receive JobStarted update
            let job_update = timeout(Duration::from_millis(100), job_update_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");

            match job_update {
                EjJobUpdate::JobStarted { nb_builders } => {
                    assert_eq!(nb_builders, 1);
                }
                _ => panic!("Expected JobStarted update, got {:?}", job_update),
            }
        });
    }

    #[tokio::test]
    async fn test_dispatch_job_with_multiple_builders() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let (job_update_tx, mut job_update_rx) = mpsc::channel(32);

            // Add multiple mock builders
            let builder_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
            let (builders_tx, mut builders_rx) = channel(16);
            for &builder_id in &builder_ids {
                let mock_builder = create_builder(builder_id, builders_tx.clone());
                dispatcher.builders.lock().await.push(mock_builder);
            }
            drop(builders_tx);

            let job = create_test_job();

            let result = dispatcher
                .dispatch_job(job, job_update_tx, Duration::from_secs(60))
                .await;
            assert!(result.is_ok());
            let job = result.unwrap();

            for _ in 0..builder_ids.len() {
                let builder_dispatch = timeout(Duration::from_millis(100), builders_rx.recv())
                    .await
                    .expect("Should receive dispatch")
                    .unwrap();
                assert_eq!(builder_dispatch, EjWsServerMessage::Build(job.clone()));
            }

            let job_update = timeout(Duration::from_millis(100), job_update_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(job_update, EjJobUpdate::JobStarted { nb_builders: 3 });
        });
    }

    #[tokio::test]
    async fn test_job_queuing_when_one_already_in_progress() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            // Add a builder
            let builder_id = Uuid::new_v4();
            let (builder_tx, _builder_rx) = channel(32);
            let mock_builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(mock_builder);

            // Dispatch first job
            let (job1_tx, mut job1_rx) = mpsc::channel(32);
            let job1 = create_test_job();
            let result1 = dispatcher
                .dispatch_job(job1, job1_tx, Duration::from_secs(60))
                .await;
            assert!(result1.is_ok());

            let update1 = timeout(Duration::from_millis(100), job1_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(update1, EjJobUpdate::JobStarted { nb_builders: 1 });

            let (job2_tx, mut job2_rx) = mpsc::channel(32);
            let job2 = create_test_job();
            let result2 = dispatcher
                .dispatch_job(job2, job2_tx, Duration::from_secs(60))
                .await;
            assert!(result2.is_ok());
            let update2 = timeout(Duration::from_millis(100), job2_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(update2, EjJobUpdate::JobAddedToQueue { queue_position: 0 })
        });
    }

    #[tokio::test]
    async fn test_job_completion_single_builder() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            // Add a builder
            let builder_id = Uuid::new_v4();
            let (builder_tx, _builder_rx) = channel(32);
            let mock_builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(mock_builder);

            let (job_tx, mut job_rx) = mpsc::channel(32);
            let job = create_test_job();
            let result = dispatcher
                .dispatch_job(job, job_tx, Duration::from_secs(60))
                .await;
            assert!(result.is_ok());
            let job = result.unwrap();

            let update = job_rx.recv().await.expect("Should receive JobStarted");
            assert_eq!(update, EjJobUpdate::JobStarted { nb_builders: 1 });

            let job_result = EjBuilderBuildResult {
                job_id: job.id,
                builder_id,
                logs: HashMap::new(),
                successful: true,
            };

            let completion_result = dispatcher.on_job_result(job_result).await;
            assert!(completion_result.is_ok());

            let update = timeout(Duration::from_millis(100), job_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(
                update,
                EjJobUpdate::BuildFinished(EjBuildResult {
                    success: true,
                    logs: Vec::new()
                })
            );
        })
    }

    #[tokio::test]
    async fn test_job_completion_multiple_builders() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let builder_ids: Vec<Uuid> = (0..3).map(|_| Uuid::new_v4()).collect();
            let (builders_tx, _builders_rx) = channel(10);
            for &builder_id in &builder_ids {
                let mock_builder = create_builder(builder_id, builders_tx.clone());
                dispatcher.builders.lock().await.push(mock_builder);
            }
            drop(builders_tx);

            // Dispatch a job
            let (job_tx, mut job_rx) = mpsc::channel(32);
            let job = create_test_job();
            let result = dispatcher
                .dispatch_job(job, job_tx, Duration::from_secs(60))
                .await;
            assert!(result.is_ok());
            let job = result.unwrap();
            let job_id = job.id;

            let update = job_rx.recv().await.expect("Should receive JobStarted");
            assert_eq!(
                update,
                EjJobUpdate::JobStarted {
                    nb_builders: builder_ids.len()
                }
            );

            for &builder_id in &builder_ids[0..2] {
                let job_result = EjBuilderBuildResult {
                    job_id,
                    builder_id,
                    successful: true,
                    logs: HashMap::new(),
                };

                let completion_result = dispatcher.on_job_result(job_result).await;
                assert!(completion_result.is_ok());

                let timeout_result = timeout(Duration::from_millis(50), job_rx.recv()).await;
                assert!(
                    timeout_result.is_err(),
                    "Should not receive JobFinished yet"
                );
            }

            // Complete job on last builder - should finish now
            let job_result = EjBuilderBuildResult {
                job_id,
                builder_id: builder_ids[2],
                logs: HashMap::new(),
                successful: true,
            };

            let completion_result = dispatcher.on_job_result(job_result).await;
            assert!(completion_result.is_ok());

            let update = timeout(Duration::from_millis(100), job_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");

            assert_eq!(
                update,
                EjJobUpdate::BuildFinished(EjBuildResult {
                    success: true,
                    logs: Vec::new()
                })
            );
        })
    }

    #[tokio::test]
    async fn test_queue_processing_after_job_completion() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let builder_id = Uuid::new_v4();
            let (builder_tx, mut builder_rx) = channel(10);
            let mock_builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(mock_builder);

            let (job1_tx, mut job1_rx) = mpsc::channel(32);
            let job1 = create_test_job();
            let result1 = dispatcher
                .dispatch_job(job1, job1_tx, Duration::from_secs(60))
                .await;
            assert!(result1.is_ok());
            let job1 = result1.unwrap();

            let (job2_tx, mut job2_rx) = mpsc::channel(32);
            let job2 = create_test_job();
            let result2 = dispatcher
                .dispatch_job(job2, job2_tx, Duration::from_secs(60))
                .await;
            assert!(result2.is_ok());
            let job2 = result2.unwrap();

            let job1_started = job1_rx.recv().await.expect("Job1 should start");
            assert_eq!(job1_started, EjJobUpdate::JobStarted { nb_builders: 1 });

            let builder_dispatch = timeout(Duration::from_millis(100), builder_rx.recv())
                .await
                .expect("Should receive dispatch")
                .unwrap();
            assert_eq!(builder_dispatch, EjWsServerMessage::Build(job1.clone()));

            let job2_queued = job2_rx.recv().await.expect("Job2 should be queued");
            assert_eq!(
                job2_queued,
                EjJobUpdate::JobAddedToQueue { queue_position: 0 }
            );

            let job1_result = EjBuilderBuildResult {
                job_id: job1.id,
                builder_id,
                successful: true,
                logs: HashMap::new(),
            };

            let completion_result = dispatcher.on_job_result(job1_result).await;
            assert!(completion_result.is_ok());

            let job1_finished = job1_rx.recv().await.expect("Job1 should finish");

            assert_eq!(
                job1_finished,
                EjJobUpdate::BuildFinished(EjBuildResult {
                    success: true,
                    logs: Vec::new()
                })
            );

            let job2_started = timeout(Duration::from_millis(100), job2_rx.recv())
                .await
                .expect("Job2 should start")
                .expect("Should have update");

            assert_eq!(job2_started, EjJobUpdate::JobStarted { nb_builders: 1 });

            let builder_dispatch = timeout(Duration::from_millis(100), builder_rx.recv())
                .await
                .expect("Should receive dispatch")
                .unwrap();
            assert_eq!(builder_dispatch, EjWsServerMessage::Build(job2.clone()));

            let job2_result = EjBuilderBuildResult {
                job_id: job2.id.clone(),
                builder_id,
                successful: true,
                logs: HashMap::new(),
            };

            let completion_result = dispatcher.on_job_result(job2_result).await;
            assert!(completion_result.is_ok());

            let job2_finished = job2_rx.recv().await.expect("Job1 should finish");

            assert_eq!(
                job2_finished,
                EjJobUpdate::BuildFinished(EjBuildResult {
                    success: true,
                    logs: Vec::new()
                })
            );
        })
    }

    #[tokio::test]
    async fn test_build_and_run_job_completion() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let builder_id = Uuid::new_v4();
            let (builder_tx, mut builder_rx) = channel(10);
            let mock_builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(mock_builder);

            // Dispatch a BuildAndRun job
            let (job_tx, mut job_rx) = mpsc::channel(32);
            let mut job = create_test_job();
            job.job_type = EjJobType::BuildAndRun;

            let result = dispatcher
                .dispatch_job(job, job_tx, Duration::from_secs(60))
                .await;
            assert!(result.is_ok());
            let job = result.unwrap();

            // Receive JobStarted
            let update = job_rx.recv().await.expect("Should receive JobStarted");
            assert_eq!(update, EjJobUpdate::JobStarted { nb_builders: 1 });

            let builder_dispatch = timeout(Duration::from_millis(100), builder_rx.recv())
                .await
                .expect("Should receive dispatch")
                .unwrap();
            assert_eq!(
                builder_dispatch,
                EjWsServerMessage::BuildAndRun(job.clone())
            );

            let job_result = EjBuilderRunResult {
                job_id: job.id,
                builder_id,
                successful: true,
                logs: HashMap::new(),
                results: HashMap::new(),
            };

            let completion_result = dispatcher.on_job_result(job_result).await;
            assert!(completion_result.is_ok());

            let job_finished = timeout(Duration::from_millis(100), job_rx.recv())
                .await
                .expect("Should receive JobFinished")
                .expect("Should have update");

            // Should also receive RunFinished for BuildAndRun jobs
            assert_eq!(
                job_finished,
                EjJobUpdate::RunFinished(EjRunResult {
                    success: true,
                    logs: Vec::new(),
                    results: Vec::new()
                })
            );
        })
    }

    #[tokio::test]
    async fn test_unexpected_job_completion() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let builder_id = Uuid::new_v4();
            let (builder_tx, _builder_rx) = channel(12);
            let mock_builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(mock_builder);

            let job_result = EjBuilderBuildResult {
                job_id: Uuid::new_v4(),
                builder_id,
                successful: true,
                logs: HashMap::new(),
            };

            let completion_result = dispatcher.on_job_result(job_result).await;
            assert!(completion_result.is_err());
        })
    }

    #[tokio::test]
    async fn test_job_timeout() {
        test!(|mut dispatcher: Dispatcher, _handle| async move {
            let (job_update_tx, mut job_update_rx) = mpsc::channel(32);

            let builder_id = Uuid::new_v4();
            let (builder_tx, mut builder_rx) = channel(32);
            let builder = create_builder(builder_id, builder_tx);
            dispatcher.builders.lock().await.push(builder);

            let job = create_test_job();

            // Dispatch the job
            let result = dispatcher
                .dispatch_job(job, job_update_tx, Duration::from_millis(100))
                .await;
            assert!(result.is_ok());
            let job = result.unwrap();

            // Get dispatch messages
            let builder_dispatch = timeout(Duration::from_millis(100), builder_rx.recv())
                .await
                .expect("Should receive dispatch")
                .unwrap();
            assert_eq!(builder_dispatch, EjWsServerMessage::Build(job.clone()));
            let job_update = timeout(Duration::from_millis(100), job_update_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(job_update, EjJobUpdate::JobStarted { nb_builders: 1 });

            // Get cancel messages
            let job_cancel = timeout(Duration::from_millis(200), job_update_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(
                job_cancel,
                EjJobUpdate::JobCancelled(EjJobCancelReason::Timeout)
            );

            let builder_cancel = timeout(Duration::from_millis(200), builder_rx.recv())
                .await
                .expect("Should receive update")
                .expect("Should have update");
            assert_eq!(
                builder_cancel,
                EjWsServerMessage::Cancel(EjJobCancelReason::Timeout, job.id)
            );
        });
    }
}
