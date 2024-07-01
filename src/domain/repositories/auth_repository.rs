use crate::model::register_user::{RegisterUserError, RegisterUserRequest};
use crate::model::user::FilteredUser;
use std::future::Future;

pub trait AuthRepository: Send + Sync + 'static {
    fn register(
        &self,
        request: &RegisterUserRequest,
    ) -> impl Future<Output = Result<FilteredUser, RegisterUserError>> + Send;

    // fn login(
    //     &self,
    //     request:
    // )
}
