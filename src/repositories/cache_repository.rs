use crate::domain::repositories::cache_repository::CacheRepository;

#[derive(Debug)]
pub struct RedisCache;

impl CacheRepository for RedisCache {}
