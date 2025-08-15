use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use crate::{BlockData, NetworkMetrics};

pub struct BlockchainAnalytics {
    db_pool: PgPool,
}

impl BlockchainAnalytics {
    pub async fn new(db_pool: &PgPool) -> Result<Self> {
        // Initialize database tables for analytics
        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS block_analytics (
                id BIGSERIAL PRIMARY KEY,
                height BIGINT NOT NULL,
                timestamp TIMESTAMPTZ NOT NULL,
                hash TEXT NOT NULL,
                difficulty DOUBLE PRECISION NOT NULL,
                tx_count INTEGER NOT NULL,
                size_bytes BIGINT NOT NULL,
                propagation_time_ms BIGINT,
                ai_processed_at TIMESTAMPTZ DEFAULT NOW(),
                UNIQUE(height, hash)
            )
        "#).execute(db_pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS network_analytics (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                peer_count INTEGER NOT NULL,
                mempool_size INTEGER NOT NULL,
                avg_block_time DOUBLE PRECISION NOT NULL,
                hashrate_estimate DOUBLE PRECISION NOT NULL,
                orphan_rate DOUBLE PRECISION NOT NULL,
                fee_percentiles JSONB,
                ai_processed_at TIMESTAMPTZ DEFAULT NOW()
            )
        "#).execute(db_pool).await?;

        sqlx::query(r#"
            CREATE TABLE IF NOT EXISTS ai_predictions (
                id BIGSERIAL PRIMARY KEY,
                timestamp TIMESTAMPTZ NOT NULL,
                prediction_type TEXT NOT NULL,
                confidence DOUBLE PRECISION NOT NULL,
                predicted_value DOUBLE PRECISION NOT NULL,
                actual_value DOUBLE PRECISION,
                accuracy DOUBLE PRECISION,
                model_version INTEGER NOT NULL DEFAULT 1
            )
        "#).execute(db_pool).await?;

        sqlx::query(r#"
            CREATE INDEX IF NOT EXISTS idx_block_analytics_height ON block_analytics(height);
            CREATE INDEX IF NOT EXISTS idx_block_analytics_timestamp ON block_analytics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_network_analytics_timestamp ON network_analytics(timestamp);
            CREATE INDEX IF NOT EXISTS idx_ai_predictions_type_time ON ai_predictions(prediction_type, timestamp);
        "#).execute(db_pool).await?;

        Ok(Self {
            db_pool: db_pool.clone(),
        })
    }

    pub async fn store_block_data(&self, block: &BlockData) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO block_analytics 
            (height, timestamp, hash, difficulty, tx_count, size_bytes, propagation_time_ms)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (height, hash) DO UPDATE SET
            propagation_time_ms = EXCLUDED.propagation_time_ms,
            ai_processed_at = NOW()
        "#)
        .bind(block.height as i64)
        .bind(block.timestamp)
        .bind(&block.hash)
        .bind(block.difficulty)
        .bind(block.tx_count as i32)
        .bind(block.size_bytes as i64)
        .bind(block.propagation_time_ms.map(|ms| ms as i64))
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn store_network_metrics(&self, metrics: &NetworkMetrics) -> Result<()> {
        let fee_percentiles_json = serde_json::to_value(&metrics.fee_percentiles)?;

        sqlx::query(r#"
            INSERT INTO network_analytics 
            (timestamp, peer_count, mempool_size, avg_block_time, hashrate_estimate, orphan_rate, fee_percentiles)
            VALUES (NOW(), $1, $2, $3, $4, $5, $6)
        "#)
        .bind(metrics.peer_count as i32)
        .bind(metrics.mempool_size as i32)
        .bind(metrics.avg_block_time)
        .bind(metrics.hashrate_estimate)
        .bind(metrics.orphan_rate)
        .bind(fee_percentiles_json)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn get_recent_data(&self, limit: u32) -> Result<Vec<BlockData>> {
        let rows = sqlx::query(r#"
            SELECT height, timestamp, hash, difficulty, tx_count, size_bytes, propagation_time_ms
            FROM block_analytics 
            ORDER BY height DESC 
            LIMIT $1
        "#)
        .bind(limit as i64)
        .fetch_all(&self.db_pool)
        .await?;

        let mut blocks = Vec::new();
        for row in rows {
            blocks.push(BlockData {
                height: row.get::<i64, _>("height") as u64,
                timestamp: row.get("timestamp"),
                hash: row.get("hash"),
                difficulty: row.get("difficulty"),
                tx_count: row.get::<i32, _>("tx_count") as u32,
                size_bytes: row.get::<i64, _>("size_bytes") as u64,
                propagation_time_ms: row.get::<Option<i64>, _>("propagation_time_ms")
                    .map(|ms| ms as u64),
            });
        }

        Ok(blocks)
    }

    pub async fn get_training_data(&self, limit: u32) -> Result<Vec<BlockData>> {
        // Get historical data for model training
        self.get_recent_data(limit).await
    }

    pub async fn store_ai_prediction(
        &self,
        prediction_type: &str,
        confidence: f64,
        predicted_value: f64,
        model_version: i32
    ) -> Result<()> {
        sqlx::query(r#"
            INSERT INTO ai_predictions 
            (timestamp, prediction_type, confidence, predicted_value, model_version)
            VALUES (NOW(), $1, $2, $3, $4)
        "#)
        .bind(prediction_type)
        .bind(confidence)
        .bind(predicted_value)
        .bind(model_version)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn update_prediction_accuracy(
        &self,
        prediction_id: i64,
        actual_value: f64
    ) -> Result<()> {
        sqlx::query(r#"
            UPDATE ai_predictions 
            SET actual_value = $1,
                accuracy = 1.0 - ABS(predicted_value - $1) / GREATEST(predicted_value, $1)
            WHERE id = $2
        "#)
        .bind(actual_value)
        .bind(prediction_id)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    pub async fn get_model_performance(&self, prediction_type: &str) -> Result<f64> {
        let row = sqlx::query(r#"
            SELECT AVG(accuracy) as avg_accuracy
            FROM ai_predictions 
            WHERE prediction_type = $1 
            AND actual_value IS NOT NULL
            AND timestamp > NOW() - INTERVAL '24 hours'
        "#)
        .bind(prediction_type)
        .fetch_one(&self.db_pool)
        .await?;

        Ok(row.get::<Option<f64>, _>("avg_accuracy").unwrap_or(0.0))
    }
}
