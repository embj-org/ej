use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use ej::ej_config::ej_board_config::EjBoardConfigApi;
use ej::ej_job::api::{EjDeployableJob, EjJob, EjJobCancelReason, EjJobType, EjJobUpdate};
use ej::ej_job::db::EjJobDb;
use ej::ej_job::logs::db::EjJobLog;
use ej::ej_job::results::api::EjJobResult;
use ej::ej_job::status::db::EjJobStatus;
use ej::ej_message::EjServerMessage;
use ej::prelude::*;
use ej::{db::connection::DbConnection, ej_connected_builder::EjConnectedBuilder};
use tokio::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    task::JoinHandle,
};
use tracing::{error, info, trace, warn};
use uuid::Uuid;

#[derive(Debug)]
pub enum DispatcherEvent {
    DispatchJob {
        job: EjDeployableJob,
        job_update_tx: Sender<EjJobUpdate>,
    },
    JobCompleted {
        job_id: Uuid,
        builder_id: Uuid,
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
}

#[derive(Debug)]
struct RunningJob {
    data: EjDeployableJob,
    tx: Sender<EjJobUpdate>,
    deployed_builders: HashSet<Uuid>,
}

impl DispatchedJob {
    pub fn new(data: EjDeployableJob, tx: Sender<EjJobUpdate>) -> Self {
        Self { data, tx }
    }
    pub fn start(self, deployed_builders: HashSet<Uuid>) -> RunningJob {
        RunningJob {
            data: self.data,
            tx: self.tx,
            deployed_builders,
        }
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

    fn start_thread(mut self, mut rx: Receiver<DispatcherEvent>) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                info!(
                    "New Dispatcher Message. Message {:?}. State {:?}",
                    message, self.state
                );
                let result = match message {
                    DispatcherEvent::DispatchJob { job, job_update_tx } => {
                        self.handle_dispatch_job(DispatchedJob::new(job, job_update_tx))
                            .await
                    }
                    DispatcherEvent::JobCompleted { job_id, builder_id } => {
                        self.handle_job_completed(job_id, builder_id).await
                    }
                };
                if let Err(err) = result {
                    error!("Error while handling last dispatcher message - {}", err);
                }
            }
        })
    }
    async fn dispatch_job_to_single_builder(
        job: EjDeployableJob,
        builder: &EjConnectedBuilder,
    ) -> bool {
        let message = if job.job_type == EjJobType::BuildAndRun {
            EjServerMessage::BuildAndRun(job)
        } else {
            EjServerMessage::Build(job)
        };
        if let Err(err) = builder.tx.send(message).await {
            error!("Failed to dispatch builder {:?} - {err}", builder);
            return false;
        }
        trace!("Builder dispatched {:?}", builder);
        return true;
    }
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
        } else {
            DispatcherPrivate::send_job_update(
                &mut job.tx,
                EjJobUpdate::JobStarted {
                    nb_builders: dispatched_builders.len(),
                },
            )
            .await;
            self.state = DispatcherState::DispatchedJob {
                job: job.start(dispatched_builders),
            };
        }
    }
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
    async fn send_job_update(tx: &Sender<EjJobUpdate>, update: EjJobUpdate) {
        if let Err(err) = tx.send(update).await {
            error!("Failed to send job update through internal channel {err}");
        }
    }

    async fn on_job_completed(job: &RunningJob, connection: &DbConnection) -> Result<()> {
        let jobdb = EjJobDb::fetch_by_id(&job.data.id, &connection)?;
        let logsdb = EjJobLog::fetch_with_board_config_by_job_id(&jobdb.id, &connection)?;
        let mut logs = Vec::new();
        for (logdb, board_config_db) in logsdb {
            let config_api =
                EjBoardConfigApi::try_from_board_config_db(board_config_db, connection)?;
            logs.push((config_api, logdb.log));
        }
        DispatcherPrivate::send_job_update(
            &job.tx,
            EjJobUpdate::JobFinished {
                success: jobdb.success(),
                logs: logs,
            },
        )
        .await;
        Ok(())
    }
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
        Ok(())
    }
}
impl Dispatcher {
    fn new(connection: DbConnection, tx: Sender<DispatcherEvent>) -> Self {
        Self {
            connection,
            builders: Arc::new(Mutex::new(Vec::new())),
            tx,
        }
    }
    pub fn create(connection: DbConnection) -> (Self, JoinHandle<()>) {
        DispatcherPrivate::create(connection)
    }

    pub async fn dispatch_job(
        &mut self,
        job: EjJob,
        job_update_tx: Sender<EjJobUpdate>,
    ) -> Result<EjDeployableJob> {
        if self.builders.lock().await.len() == 0 {
            return Err(Error::NoBuildersAvailable);
        }
        let job = job.create(&mut self.connection)?;

        self.tx
            .send(DispatcherEvent::DispatchJob {
                job: job.clone(),
                job_update_tx,
            })
            .await
            .map_err(|err| {
                error!("Failed to send dispatcher event {err}");
                Error::ChannelSendError
            })?;

        Ok(job)
    }

    pub async fn on_job_result(&mut self, result: impl EjJobResult) -> Result<()> {
        let job_id = result.job_id();
        let builder_id = result.builder_id();
        result.save(&mut self.connection)?;

        self.tx
            .send(DispatcherEvent::JobCompleted {
                job_id: job_id,
                builder_id: builder_id,
            })
            .await
            .map_err(|err| {
                error!("Failed to send dispatcher event {err}");
                Error::ChannelSendError
            })?;

        Ok(())
    }
}
