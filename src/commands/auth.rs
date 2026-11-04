//! `auth status`. This CLI has no credentials, and the command exists because SPEC v1 §1.2
//! requires it of every family member, so a driver can read `required: false` uniformly.

use pk_cli_auth::AuthStatus;

use crate::error::AppError;

pub fn status(json: bool) -> Result<(), AppError> {
    AuthStatus::not_required().emit(json);
    Ok(())
}
