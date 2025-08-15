import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  scenarios: {
    constant_load: {
      executor: 'constant-vus',
      vus: 50,           // virtual users
      duration: '60s',
    },
  },
  thresholds: {
    // Hard budgets — fail CI if exceeded
    http_req_failed: ['rate==0'],                           // zero errors
    'http_req_duration{endpoint:score}': ['p(95)<200'],     // p95 < 200ms
    'http_req_duration{endpoint:fee}':   ['p(95)<150'],     // p95 < 150ms
  },
};

const BASE = __ENV.AI_URL || 'http://localhost:8081';

export default function () {
  const scorePayload = JSON.stringify({
    orphan_rate: 0.01, reorgs_24h: 0, mempool_tx: 1200,
    mean_block_interval: 610, top_miner_share: 0.22, peer_churn_rate: 0.05
  });
  const feePayload = JSON.stringify({ size_vb: 250, fee_sat: 500, age_s: 30 });
  const params = { headers: { 'Content-Type': 'application/json' } };

  let r1 = http.post(`${BASE}/score/anomaly`, scorePayload, params);
  check(r1, { 'score 200': (r)=> r.status === 200 }, { endpoint: 'score' });

  let r2 = http.post(`${BASE}/hint/fee`, feePayload, params);
  check(r2, { 'fee 200': (r)=> r.status === 200 }, { endpoint: 'fee' });

  sleep(0.1);
}
