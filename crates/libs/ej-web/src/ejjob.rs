use ej_dispatcher_sdk::ejjob::{
    EjDeployableJob, EjJob, EjJobType,
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
