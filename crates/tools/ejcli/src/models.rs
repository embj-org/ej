use ej::ej_job::api::EjJob;

use crate::cli::DispatchArgs;

impl From<DispatchArgs> for EjJob {
    fn from(value: DispatchArgs) -> Self {
        Self {
            remote_token: value.remote_token,
            commit_hash: value.commit_hash,
            remote_url: value.remote_url,
        }
    }
}
