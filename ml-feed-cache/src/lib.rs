use std::time::{SystemTime, UNIX_EPOCH};

use consts::{
    MAX_GLOBAL_CACHE_LEN, MAX_SUCCESS_HISTORY_CACHE_LEN, MAX_USER_CACHE_LEN,
    MAX_WATCH_HISTORY_CACHE_LEN,
};
use redis::AsyncCommands;
use types::{MLFeedCacheHistoryItem, PostItem, get_history_item_score};

pub mod consts;
pub mod types;

pub type RedisPool = bb8::Pool<bb8_redis::RedisConnectionManager>;

#[derive(Clone)]
pub struct MLFeedCacheState {
    pub redis_pool: RedisPool,
}

pub async fn init_redis() -> RedisPool {
    let redis_url =
        std::env::var("ML_FEED_CACHE_REDIS_URL").expect("ML_FEED_CACHE_REDIS_URL must be set");

    let manager = bb8_redis::RedisConnectionManager::new(redis_url.clone())
        .expect("failed to open connection to redis");
    RedisPool::builder().build(manager).await.unwrap()
}

impl MLFeedCacheState {
    pub async fn new() -> Self {
        let redis_pool = init_redis().await;
        Self { redis_pool }
    }

    pub async fn add_user_watch_history_items(
        &self,
        key: &str,
        items: Vec<MLFeedCacheHistoryItem>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();

        let items = items
            .iter()
            .map(|item| (get_history_item_score(item), item.clone()))
            .collect::<Vec<_>>();

        // zadd_multiple in groups of 1000
        let chunk_size = 1000;
        for chunk in items.chunks(chunk_size) {
            let _res = conn
                .zadd_multiple::<&str, f64, MLFeedCacheHistoryItem, ()>(key, chunk)
                .await?;
        }

        // get num items in the list
        let num_items = conn.zcard::<&str, u64>(key).await?;

        // if num items is greater than 5000, remove the oldest items till len is 5000 without while loop
        if num_items > MAX_WATCH_HISTORY_CACHE_LEN {
            let _res = conn
                .zremrangebyrank::<&str, ()>(
                    key,
                    0,
                    (num_items - (MAX_WATCH_HISTORY_CACHE_LEN + 1)) as isize,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn add_user_success_history_items(
        &self,
        key: &str,
        items: Vec<MLFeedCacheHistoryItem>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();

        let items = items
            .iter()
            .map(|item| (get_history_item_score(item), item.clone()))
            .collect::<Vec<_>>();

        // zadd_multiple in groups of 1000
        let chunk_size = 1000;
        for chunk in items.chunks(chunk_size) {
            let _res = conn
                .zadd_multiple::<&str, f64, MLFeedCacheHistoryItem, ()>(key, chunk)
                .await?;
        }

        // get num items in the list
        let num_items = conn.zcard::<&str, u64>(key).await?;

        if num_items > MAX_SUCCESS_HISTORY_CACHE_LEN {
            let _res = conn
                .zremrangebyrank::<&str, ()>(
                    key,
                    0,
                    (num_items - (MAX_SUCCESS_HISTORY_CACHE_LEN + 1)) as isize,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn get_history_items(
        &self,
        key: &str,
        start: u64,
        end: u64,
    ) -> Result<Vec<MLFeedCacheHistoryItem>, anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();

        let items = conn
            .zrevrange::<&str, Vec<MLFeedCacheHistoryItem>>(key, start as isize, end as isize)
            .await?;

        Ok(items)
    }

    pub async fn get_history_items_len(&self, key: &str) -> Result<u64, anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();
        let num_items = conn.zcard::<&str, u64>(key).await?;
        Ok(num_items)
    }

    pub async fn add_user_cache_items(
        &self,
        key: &str,
        items: Vec<PostItem>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;

        let items = items
            .iter()
            .map(|item| (timestamp, item.clone()))
            .collect::<Vec<_>>();

        // zadd_multiple in groups of 1000
        let chunk_size = 1000;
        for chunk in items.chunks(chunk_size) {
            let _res = conn
                .zadd_multiple::<&str, f64, PostItem, ()>(key, chunk)
                .await?;
        }

        // get num items in the list
        let num_items = conn.zcard::<&str, u64>(key).await?;

        if num_items > MAX_USER_CACHE_LEN {
            let _res = conn
                .zremrangebyrank::<&str, ()>(key, 0, (num_items - MAX_USER_CACHE_LEN - 1) as isize)
                .await?;
        }

        Ok(())
    }

    pub async fn add_global_cache_items(
        &self,
        key: &str,
        items: Vec<PostItem>,
    ) -> Result<(), anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as f64;

        let items = items
            .iter()
            .map(|item| (timestamp, item.clone()))
            .collect::<Vec<_>>();

        // zadd_multiple in groups of 1000
        let chunk_size = 1000;
        for chunk in items.chunks(chunk_size) {
            let _res = conn
                .zadd_multiple::<&str, f64, PostItem, ()>(key, chunk)
                .await?;
        }

        // get num items in the list
        let num_items = conn.zcard::<&str, u64>(key).await?;

        if num_items > MAX_GLOBAL_CACHE_LEN {
            let _res = conn
                .zremrangebyrank::<&str, ()>(
                    key,
                    0,
                    (num_items - MAX_GLOBAL_CACHE_LEN - 1) as isize,
                )
                .await?;
        }

        Ok(())
    }

    pub async fn get_cache_items(
        &self,
        key: &str,
        start: u64,
        end: u64,
    ) -> Result<Vec<PostItem>, anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();

        let items = conn
            .zrevrange::<&str, Vec<PostItem>>(key, start as isize, end as isize)
            .await?;

        Ok(items)
    }

    pub async fn get_cache_items_len(&self, key: &str) -> Result<u64, anyhow::Error> {
        let mut conn = self.redis_pool.get().await.unwrap();
        let num_items = conn.zcard::<&str, u64>(key).await?;
        Ok(num_items)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_add_user_watch_history_items() {
        let state = MLFeedCacheState::new().await;

        let mut conn = state.redis_pool.get().await.unwrap();
        // delete the key
        let _res = conn.del::<&str, ()>("test_key").await;
        assert!(_res.is_ok());

        let num_items = conn.zcard::<&str, u64>("test_key").await.unwrap();
        assert_eq!(num_items, 0);

        let mut items = Vec::new();
        for i in 0..5010 {
            items.push(MLFeedCacheHistoryItem {
                video_id: format!("test_video_id{}", i),
                item_type: "video_viewed".to_string(),
                canister_id: "test_canister_id".to_string(),
                post_id: i as u64,
                nsfw_probability: 0.0,
                timestamp: SystemTime::now(),
                percent_watched: i as f32 / 100.0,
            });
        }

        let res = state.add_user_watch_history_items("test_key", items).await;
        assert!(res.is_ok());

        let num_items = conn.zcard::<&str, u64>("test_key").await.unwrap();
        assert_eq!(num_items, 5000);

        let items = conn
            .zrevrange_withscores::<&str, Vec<(MLFeedCacheHistoryItem, f64)>>("test_key", 0, 4)
            .await
            .unwrap();
        assert_eq!(items.len(), 5);

        // print the items
        for item in items {
            println!("{:?}", item);
        }

        // delete the key
        let _res = conn.del::<&str, ()>("test_key").await;
        assert!(_res.is_ok());

        let num_items = conn.zcard::<&str, u64>("test_key").await.unwrap();
        assert_eq!(num_items, 0);
    }

    #[tokio::test]
    async fn test_add_user_success_history_items() {
        let state = MLFeedCacheState::new().await;

        let mut conn = state.redis_pool.get().await.unwrap();
        // delete the key
        let _res = conn.del::<&str, ()>("test_key").await;
        assert!(_res.is_ok());

        let num_items = conn.zcard::<&str, u64>("test_key").await.unwrap();
        assert_eq!(num_items, 0);

        let mut items = Vec::new();
        for i in 0..10100 {
            items.push(MLFeedCacheHistoryItem {
                video_id: format!("test_video_id{}", i),
                item_type: "like_video".to_string(),
                canister_id: "test_canister_id".to_string(),
                post_id: i as u64,
                nsfw_probability: 0.0,
                timestamp: SystemTime::now() + Duration::from_secs(i * 100 as u64),
                percent_watched: 0.0,
            });
        }

        let res = state
            .add_user_success_history_items("test_key", items)
            .await;
        assert!(res.is_ok());

        let num_items = conn.zcard::<&str, u64>("test_key").await.unwrap();
        assert_eq!(num_items, 10000);

        let items = conn
            .zrevrange_withscores::<&str, Vec<(MLFeedCacheHistoryItem, f64)>>("test_key", 0, 4)
            .await
            .unwrap();
        assert_eq!(items.len(), 5);

        // print the items
        for item in items {
            println!("{:?}", item);
        }

        // delete the key
        let _res = conn.del::<&str, ()>("test_key").await;
        assert!(_res.is_ok());

        let num_items = conn.zcard::<&str, u64>("test_key").await.unwrap();
        assert_eq!(num_items, 0);
    }
}
