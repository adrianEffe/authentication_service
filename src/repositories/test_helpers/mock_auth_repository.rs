#[cfg(test)]
pub mod test_helpers {
    use anyhow::anyhow;
    use std::{mem, ops::DerefMut, sync::Arc};
    use tokio::sync::Mutex;

    use crate::domain::{
        model::{
            auth::AuthorizationError,
            login_user::{LoginUserError, LoginUserRequest},
            register_user::{HashedUserPassword, RegisterUserError, RegisterUserRequest},
            user::{FilteredUser, User},
            user_email::UserEmail,
            user_id::UserId,
            user_password::UserPassword,
        },
        repositories::auth_repository::AuthRepository,
    };

    pub struct MockAuthRepository {
        /// It would be great for result to just take a Result instead of the below, unfortunately
        /// it needs to conform to `Clone` but RegisterUserError` has an `Unknown` variant that
        /// might wrap errors that are not Clone.
        pub register_result: Arc<Mutex<Result<FilteredUser, RegisterUserError>>>,
        pub auth_result: Arc<Mutex<Result<User, AuthorizationError>>>,
        pub login_result: Arc<Mutex<Result<User, LoginUserError>>>,
    }

    impl AuthRepository for MockAuthRepository {
        async fn register(
            &self,
            _request: &RegisterUserRequest,
        ) -> Result<FilteredUser, RegisterUserError> {
            let mut guard = self.register_result.lock().await;
            let mut result = Err(RegisterUserError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }

        async fn auth(&self, _request: &UserId) -> Result<User, AuthorizationError> {
            let mut guard = self.auth_result.lock().await;
            let mut result = Err(AuthorizationError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }

        async fn login(&self, _request: &LoginUserRequest) -> Result<User, LoginUserError> {
            let mut guard = self.login_result.lock().await;
            let mut result = Err(LoginUserError::Unknown(anyhow!("substitute error")));
            mem::swap(guard.deref_mut(), &mut result);
            result
        }
    }

    impl MockAuthRepository {
        pub fn success(email: &str, password: &str) -> MockAuthRepository {
            let user = User::new(email, password);
            let filtered_user = FilteredUser::from(&user);
            let register_result = Arc::new(Mutex::new(Ok(filtered_user)));
            let auth_result = Arc::new(Mutex::new(Ok(user.clone())));
            let login_result = Arc::new(Mutex::new(Ok(user)));

            MockAuthRepository {
                register_result,
                auth_result,
                login_result,
            }
        }

        pub fn failure() -> MockAuthRepository {
            let register_result = Arc::new(Mutex::new(Err(RegisterUserError::Unknown(anyhow!(
                "register result error"
            )))));
            let auth_result = Arc::new(Mutex::new(Err(AuthorizationError::Unknown(anyhow!(
                "auth result error"
            )))));
            let login_result = Arc::new(Mutex::new(Err(LoginUserError::Unknown(anyhow!(
                "login result error"
            )))));

            MockAuthRepository {
                register_result,
                auth_result,
                login_result,
            }
        }
    }

    #[tokio::test]
    async fn test_register_success() {
        let email = "adrian@email.com";
        let password = "password";

        let mock_repo = MockAuthRepository::success(email, password);

        let result = mock_repo
            .register(&RegisterUserRequest::new(
                UserEmail::new(email).unwrap(),
                HashedUserPassword::new(UserPassword::new(password).unwrap()).unwrap(),
            ))
            .await;

        assert_eq!(email.to_string(), result.unwrap().email);
    }

    #[tokio::test]
    async fn test_register_failure() {
        let email = "adrian@email.com";
        let password = "password";

        let mock_repo = MockAuthRepository::failure();

        let result = mock_repo
            .register(&RegisterUserRequest::new(
                UserEmail::new(email).unwrap(),
                HashedUserPassword::new(UserPassword::new(password).unwrap()).unwrap(),
            ))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_auth_success() {
        let email = "adrian@email.com";
        let password = "password";
        let uuid = uuid::Uuid::new_v4();
        let user_id = UserId::new(uuid);

        let mock_repo = MockAuthRepository::success(email, password);

        let result = mock_repo.auth(&user_id).await;

        assert_eq!(email, &result.unwrap().email);
    }

    #[tokio::test]
    async fn test_auth_failure() {
        let uuid = uuid::Uuid::new_v4();
        let user_id = UserId::new(uuid);

        let mock_repo = MockAuthRepository::failure();

        let result = mock_repo.auth(&user_id).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_login_success() {
        let email = "adrian@email.com";
        let password = "password";

        let mock_repo = MockAuthRepository::success(email, password);

        let result = mock_repo
            .login(&LoginUserRequest::new(
                UserEmail::new(email).unwrap(),
                UserPassword::new(password).unwrap(),
            ))
            .await;

        assert_eq!(email, result.unwrap().email);
    }

    #[tokio::test]
    async fn test_login_failure() {
        let email = "adrian@email.com";
        let password = "password";

        let mock_repo = MockAuthRepository::failure();

        let result = mock_repo
            .login(&LoginUserRequest::new(
                UserEmail::new(email).unwrap(),
                UserPassword::new(password).unwrap(),
            ))
            .await;

        assert!(result.is_err())
    }
}
