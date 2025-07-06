//! Job management utilities for web handlers.

use ej_dispatcher_sdk::ejjob::{
    EjDeployableJob, EjJob, EjJobApi, EjJobType,
    results::{EjBuilderBuildResult, EjBuilderRunResult},
};
use ej_models::{
    db::connection::DbConnection,
    job::{
        ejjob::{EjJobCreate, EjJobDb},
        ejjob_logs::EjJobLogCreate,
        ejjob_results::EjJobResultCreate,
        ejjob_status::EjJobStatus,
    },
};
use uuid::Uuid;

use crate::{error::Error, prelude::*, traits::job_result::EjJobResult};

/// Creates a new job from the provided job data.
///
/// Converts an `EjJob` into a database record and returns a `EjDeployableJob`
/// that can be dispatched to builders.
///
/// # Examples
///
/// ```rust
/// use ej_web::ejjob::create_job;
/// use ej_dispatcher_sdk::ejjob::{EjJob, EjJobType};
/// # use ej_models::db::connection::DbConnection;
///
/// # async fn example(mut connection: DbConnection) -> Result<(), Box<dyn std::error::Error>> {
/// let job = EjJob {
///     commit_hash: "abc123def456".to_string(),
///     remote_url: "https://github.com/user/repo.git".to_string(),
///     remote_token: Some("github_token".to_string()),
///     job_type: EjJobType::Build,
/// };
///
/// let deployable_job = create_job(job, &mut connection)?;
/// println!("Created job with ID: {}", deployable_job.id);
/// # Ok(())
/// # }
/// ```
pub fn create_job(ejjob: EjJob, connection: &mut DbConnection) -> Result<EjDeployableJob> {
    let job = EjJobCreate {
        commit_hash: ejjob.commit_hash,
        remote_url: ejjob.remote_url,
        job_type: ejjob.job_type as i32,
    };
    let job = job.save(connection)?;

    Ok(EjDeployableJob {
        id: job.id,
        job_type: job.job_type.into(),
        commit_hash: job.commit_hash,
        remote_url: job.remote_url,
        remote_token: ejjob.remote_token,
    })
}

impl From<EjJobDb> for W<EjJobApi> {
    fn from(value: EjJobDb) -> Self {
        Self(EjJobApi {
            id: value.id,
            commit_hash: value.commit_hash,
            remote_url: value.remote_url,
            job_type: value.job_type.into(),
            status: value.status.into(),
            dispatched_at: value.dispatched_at,
            finished_at: value.finished_at,
        })
    }
}

/// Implementation of EjJobResult for build job results.
///
/// Saves build job results including logs and status updates to the database.
///
/// # Examples
///
/// ```rust
/// use ej_web::traits::job_result::EjJobResult;
/// use ej_dispatcher_sdk::ejjob::results::EjBuilderBuildResult;
/// use std::collections::HashMap;
/// use uuid::Uuid;
/// # use ej_models::db::connection::DbConnection;
///
/// # async fn example(connection: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
/// let build_result = EjBuilderBuildResult {
///     job_id: Uuid::new_v4(),
///     builder_id: Uuid::new_v4(),
///     successful: true,
///     logs: HashMap::new(),
/// };
///
/// build_result.save(connection)?;
/// # Ok(())
/// # }
/// ```
impl EjJobResult for EjBuilderBuildResult {
    fn save(self, connection: &DbConnection) -> Result<()> {
        let result = self;
        let job = EjJobDb::fetch_by_id(&result.job_id, connection)?;
        let job_type: EjJobType = job.fetch_type(connection)?.id.into();
        if job_type != EjJobType::Build {
            return Err(Error::InvalidJobType);
        }

        let job_status = if result.successful {
            EjJobStatus::success()
        } else {
            EjJobStatus::failed()
        };
        job.update_status(job_status, connection)?;

        for (board_config_id, logs) in result.logs.iter() {
            let log = EjJobLogCreate {
                ejjob_id: result.job_id.clone(),
                ejboard_config_id: *board_config_id,
                log: logs.join(""),
            };
            log.save(connection)?;
        }
        Ok(())
    }

    fn job_id(&self) -> Uuid {
        self.job_id
    }

    fn builder_id(&self) -> Uuid {
        self.builder_id
    }
}

/// Implementation of EjJobResult for run job results.
///
/// Saves run job results including logs, execution results, and status updates to the database.
///
/// # Examples
///
/// ```rust
/// use ej_web::traits::job_result::EjJobResult;
/// use ej_dispatcher_sdk::ejjob::results::EjBuilderRunResult;
/// use std::collections::HashMap;
/// use uuid::Uuid;
/// # use ej_models::db::connection::DbConnection;
///
/// # async fn example(connection: &DbConnection) -> Result<(), Box<dyn std::error::Error>> {
/// let run_result = EjBuilderRunResult {
///     job_id: Uuid::new_v4(),
///     builder_id: Uuid::new_v4(),
///     successful: true,
///     logs: HashMap::new(),
///     results: HashMap::new(),
/// };
///
/// run_result.save(connection)?;
/// # Ok(())
/// # }
/// ```
impl EjJobResult for EjBuilderRunResult {
    fn save(self, connection: &DbConnection) -> Result<()> {
        let run_result = self;
        let job = EjJobDb::fetch_by_id(&run_result.job_id, connection)?;
        let job_type: EjJobType = job.fetch_type(connection)?.id.into();
        if job_type != EjJobType::BuildAndRun {
            return Err(Error::InvalidJobType);
        }

        let job_status = if run_result.successful {
            EjJobStatus::success()
        } else {
            EjJobStatus::failed()
        };
        job.update_status(job_status, connection)?;

        for (board_config_id, logs) in run_result.logs.iter() {
            let logs = EjJobLogCreate {
                ejjob_id: run_result.job_id.clone(),
                ejboard_config_id: *board_config_id,
                log: logs.join(""),
            };
            logs.save(connection)?;
        }

        for (board_config_id, result) in run_result.results.iter() {
            let result = EjJobResultCreate {
                ejjob_id: run_result.job_id.clone(),
                ejboard_config_id: *board_config_id,
                result: result.to_string(),
            };
            result.save(connection)?;
        }
        Ok(())
    }

    fn job_id(&self) -> Uuid {
        self.job_id
    }

    fn builder_id(&self) -> Uuid {
        self.builder_id
    }
}
