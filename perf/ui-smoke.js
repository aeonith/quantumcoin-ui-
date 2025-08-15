import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  vus: 10,
  duration: '30s',
  thresholds: {
    http_req_failed: ['rate==0'],
    http_req_duration: ['p(95)<300'],
  },
};

const UI = __ENV.UI_URL || 'http://localhost:3000';

export default function () {
  const r = http.get(`${UI}/api/health`);
  check(r, { 'health 200': (res)=> res.status === 200 });
  sleep(0.2);
}
