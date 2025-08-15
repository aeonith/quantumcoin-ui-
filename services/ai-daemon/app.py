from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import os
import joblib
import numpy as np
from typing import List, Optional
import logging

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="QuantumCoin AI Sentinel", version="1.0.0")

# Load ML model if available
MODEL_PATH = os.getenv("MODEL_PATH", "model.joblib")
model = None
if os.path.exists(MODEL_PATH):
    try:
        model = joblib.load(MODEL_PATH)
        logger.info(f"Loaded ML model from {MODEL_PATH}")
    except Exception as e:
        logger.warning(f"Failed to load model: {e}")

class NetworkTelemetry(BaseModel):
    orphan_rate: float
    reorgs_24h: int
    mempool_tx: int
    mean_block_interval: float
    top_miner_share: float
    peer_churn_rate: float

class TransactionHint(BaseModel):
    size_vb: int
    fee_sat: int
    age_s: int

class RiskScore(BaseModel):
    risk_score: float
    confidence: float
    reasons: List[str]

class FeeHint(BaseModel):
    sats_per_vbyte: int
    p_confirm_1: float
    p_confirm_3: float

@app.get("/health")
def health():
    """Health check endpoint"""
    return {
        "status": "operational",
        "model_loaded": model is not None,
        "version": "1.0.0",
        "service": "ai-sentinel"
    }

@app.post("/score/anomaly", response_model=RiskScore)
def score_network_risk(telemetry: NetworkTelemetry):
    """Score network anomaly risk based on telemetry"""
    try:
        # Safe baseline scoring without ML model
        concentration_risk = telemetry.top_miner_share * 50
        orphan_risk = telemetry.orphan_rate * 200
        reorg_risk = telemetry.reorgs_24h * 5
        mempool_risk = max(0, (telemetry.mempool_tx - 5000) / 100)
        
        raw_score = concentration_risk + orphan_risk + reorg_risk + mempool_risk
        risk_score = max(0, min(100, raw_score))
        
        reasons = []
        if telemetry.top_miner_share > 0.3:
            reasons.append("mining_concentration")
        if telemetry.orphan_rate > 0.02:
            reasons.append("high_orphan_rate")
        if telemetry.reorgs_24h > 1:
            reasons.append("frequent_reorgs")
        if telemetry.mempool_tx > 10000:
            reasons.append("mempool_congestion")
            
        return RiskScore(
            risk_score=risk_score,
            confidence=0.8 if model else 0.6,
            reasons=reasons or ["normal_operation"]
        )
    except Exception as e:
        logger.error(f"Risk scoring error: {e}")
        raise HTTPException(status_code=500, detail="Risk scoring failed")

@app.post("/hint/fee", response_model=FeeHint)
def suggest_transaction_fee(tx: TransactionHint):
    """Suggest optimal transaction fee"""
    try:
        # Smart baseline fee calculation
        base_rate = 2
        size_multiplier = max(1, tx.size_vb / 250)
        age_penalty = max(1, tx.age_s / 3600)  # Penalty for old transactions
        
        suggested_rate = int(base_rate * size_multiplier * age_penalty)
        suggested_rate = max(1, min(suggested_rate, 50))  # Cap at 50 sats/vB
        
        # Confidence estimates
        p_confirm_1 = 0.35 + (suggested_rate / 100)
        p_confirm_3 = 0.8 + (suggested_rate / 200)
        p_confirm_1 = min(0.95, p_confirm_1)
        p_confirm_3 = min(0.99, p_confirm_3)
        
        return FeeHint(
            sats_per_vbyte=suggested_rate,
            p_confirm_1=p_confirm_1,
            p_confirm_3=p_confirm_3
        )
    except Exception as e:
        logger.error(f"Fee hint error: {e}")
        raise HTTPException(status_code=500, detail="Fee estimation failed")

@app.get("/metrics")
def get_metrics():
    """Prometheus-style metrics endpoint"""
    return {
        "ai_model_loaded": 1 if model else 0,
        "requests_processed": "counter_placeholder",
        "service_uptime": "gauge_placeholder",
        "memory_usage": "gauge_placeholder"
    }

if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")
