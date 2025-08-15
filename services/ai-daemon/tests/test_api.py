from fastapi.testclient import TestClient
from app import app
import pytest

client = TestClient(app)

def test_health():
    """Test health check endpoint"""
    response = client.get("/health")
    assert response.status_code == 200
    data = response.json()
    assert "status" in data
    assert "model_loaded" in data
    assert "version" in data
    assert data["status"] == "operational"

def test_score_anomaly():
    """Test anomaly scoring endpoint"""
    payload = {
        "orphan_rate": 0.01,
        "reorgs_24h": 0,
        "mempool_tx": 1000,
        "mean_block_interval": 610,
        "top_miner_share": 0.2,
        "peer_churn_rate": 0.05
    }
    response = client.post("/score/anomaly", json=payload)
    assert response.status_code == 200
    data = response.json()
    assert "risk_score" in data
    assert "confidence" in data
    assert "reasons" in data
    assert 0 <= data["risk_score"] <= 100
    assert 0 <= data["confidence"] <= 1

def test_fee_hint():
    """Test fee hint endpoint"""
    payload = {
        "size_vb": 250,
        "fee_sat": 500,
        "age_s": 10
    }
    response = client.post("/hint/fee", json=payload)
    assert response.status_code == 200
    data = response.json()
    assert "sats_per_vbyte" in data
    assert "p_confirm_1" in data  
    assert "p_confirm_3" in data
    assert data["sats_per_vbyte"] >= 1
    assert 0 <= data["p_confirm_1"] <= 1
    assert 0 <= data["p_confirm_3"] <= 1

def test_metrics():
    """Test metrics endpoint"""
    response = client.get("/metrics")
    assert response.status_code == 200
    data = response.json()
    assert "ai_model_loaded" in data

def test_invalid_anomaly_request():
    """Test invalid anomaly scoring request"""
    payload = {"invalid": "data"}
    response = client.post("/score/anomaly", json=payload)
    assert response.status_code == 422  # Validation error

def test_invalid_fee_request():
    """Test invalid fee hint request"""
    payload = {"invalid": "data"}
    response = client.post("/hint/fee", json=payload)
    assert response.status_code == 422  # Validation error
