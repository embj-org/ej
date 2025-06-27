use crate::ej_job::{api::EjJobType, job_type::db::EjJobTypeDb};

pub mod db;

impl From<i32> for EjJobType {
    fn from(value: i32) -> Self {
        match value {
            0 => EjJobType::Build,
            1 => EjJobType::BuildAndRun,
            _ => unreachable!(),
        }
    }
}

impl From<EjJobTypeDb> for EjJobType {
    fn from(value: EjJobTypeDb) -> Self {
        value.id.into()
    }
}
