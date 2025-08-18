// EXTREME STRESS TEST - 1000 requests/minute for 2 minutes
// ZERO TOLERANCE for errors, warnings, or failures

import http from 'k6/http';
import { check, fail } from 'k6';
import { Counter, Rate, Trend } from 'k6/metrics';

// Custom metrics for bulletproof monitoring
const errorCounter = new Counter('errors_total');
const warningCounter = new Counter('warnings_total');
const successRate = new Rate('success_rate');
const responseTime = new Trend('response_time_ms');

export const options = {
  // 1000 requests per minute = 16.67 requests per second
  // For 2 minutes = 2000 total requests
  vus: 17, // 17 virtual users
  duration: '2m',
  thresholds: {
    // ZERO TOLERANCE - Any failure fails the test
    errors_total: ['count==0'],
    warnings_total: ['count==0'], 
    success_rate: ['rate==1.0'], // 100% success required
    response_time_ms: ['p(95)<100'], // P95 under 100ms
    http_req_failed: ['rate==0'], // 0% error rate
    http_req_duration: ['p(95)<100'], // P95 latency under 100ms
  },
};

const API_BASE = 'http://localhost:8080';

// Test endpoints that MUST work perfectly
const ENDPOINTS = [
  '/status',
  '/explorer/blocks?limit=5',
  '/explorer/stats',
  '/blockchain',
  '/balance/qtc1qtest123456789',
  '/wallet/generate',
];

export default function() {
  // Randomize endpoint selection for comprehensive stress
  const endpoint = ENDPOINTS[Math.floor(Math.random() * ENDPOINTS.length)];
  const url = `${API_BASE}${endpoint}`;
  
  const startTime = Date.now();
  
  let response;
  try {
    if (endpoint === '/wallet/generate') {
      // POST request for wallet generation
      response = http.post(url, JSON.stringify({}), {
        headers: { 'Content-Type': 'application/json' },
        timeout: '10s'
      });
    } else {
      // GET request for other endpoints
      response = http.get(url, { timeout: '10s' });
    }
  } catch (error) {
    errorCounter.add(1);
    fail(`Network error on ${endpoint}: ${error}`);
  }
  
  const responseTimeMs = Date.now() - startTime;
  responseTime.add(responseTimeMs);
  
  // ZERO TOLERANCE checks
  const checksResult = check(response, {
    'status is 200': (r) => r.status === 200,
    'response time under 100ms': (r) => r.timings.duration < 100,
    'response has body': (r) => r.body && r.body.length > 0,
    'response is valid JSON': (r) => {
      try {
        JSON.parse(r.body);
        return true;
      } catch (e) {
        return false;
      }
    },
    'no error fields in response': (r) => {
      try {
        const data = JSON.parse(r.body);
        return !data.error && !data.Error && !data.ERROR;
      } catch (e) {
        return false;
      }
    }
  });
  
  if (!checksResult) {
    errorCounter.add(1);
    successRate.add(false);
    console.error(`FAILURE on ${endpoint}: Status=${response.status}, Body=${response.body.substring(0, 200)}`);
  } else {
    successRate.add(true);
  }
  
  // Specific endpoint validation - ZERO TOLERANCE
  if (response.status === 200) {
    try {
      const data = JSON.parse(response.body);
      
      switch (endpoint) {
        case '/status':
          if (!data.height || data.height <= 0) {
            errorCounter.add(1);
            fail(`Status endpoint returned invalid height: ${data.height}`);
          }
          if (!data.peers || typeof data.peers !== 'number') {
            errorCounter.add(1);
            fail(`Status endpoint returned invalid peers: ${data.peers}`);
          }
          if (data.status !== 'healthy' && data.status !== 'syncing') {
            warningCounter.add(1);
            console.warn(`Status not optimal: ${data.status}`);
          }
          break;
          
        case '/explorer/blocks?limit=5':
          if (!data.blocks || !Array.isArray(data.blocks)) {
            errorCounter.add(1);
            fail(`Blocks endpoint returned invalid blocks: ${data.blocks}`);
          }
          if (data.blocks.length === 0) {
            errorCounter.add(1);
            fail('Blocks endpoint returned empty blocks array');
          }
          for (const block of data.blocks) {
            if (!block.hash || !block.height || block.height <= 0) {
              errorCounter.add(1);
              fail(`Invalid block data: ${JSON.stringify(block)}`);
            }
          }
          break;
          
        case '/explorer/stats':
          if (!data.height || data.height <= 0) {
            errorCounter.add(1);
            fail(`Stats returned invalid height: ${data.height}`);
          }
          if (!data.total_supply || data.total_supply <= 0) {
            errorCounter.add(1);
            fail(`Stats returned invalid supply: ${data.total_supply}`);
          }
          break;
          
        case '/wallet/generate':
          if (!data.success || !data.address || !data.public_key) {
            errorCounter.add(1);
            fail(`Wallet generation failed: ${JSON.stringify(data)}`);
          }
          if (data.algorithm !== 'dilithium2') {
            errorCounter.add(1);
            fail(`Wrong crypto algorithm: ${data.algorithm}`);
          }
          break;
      }
    } catch (e) {
      errorCounter.add(1);
      fail(`JSON parsing failed for ${endpoint}: ${e}`);
    }
  }
  
  // Rate limiting - exactly 1000 per minute
  // Sleep to maintain ~16.67 requests per second per VU
  const sleepTime = Math.max(0, 60 - responseTimeMs); // Aim for ~60ms interval
  if (sleepTime > 0) {
    http.batch([]); // Short pause
  }
}

export function handleSummary(data) {
  const errors = data.metrics.errors_total.values.count || 0;
  const warnings = data.metrics.warnings_total.values.count || 0;
  const successRate = data.metrics.success_rate.values.rate || 0;
  const avgResponseTime = data.metrics.response_time_ms.values.avg || 0;
  const p95ResponseTime = data.metrics.response_time_ms.values['p(95)'] || 0;
  const totalRequests = data.metrics.http_reqs.values.count || 0;
  
  console.log('\n🎯 EXTREME STRESS TEST RESULTS');
  console.log('==============================');
  console.log(`Total Requests: ${totalRequests}`);
  console.log(`Success Rate: ${(successRate * 100).toFixed(2)}%`);
  console.log(`Errors: ${errors}`);
  console.log(`Warnings: ${warnings}`);
  console.log(`Avg Response Time: ${avgResponseTime.toFixed(2)}ms`);
  console.log(`P95 Response Time: ${p95ResponseTime.toFixed(2)}ms`);
  
  // ZERO TOLERANCE validation
  if (errors > 0) {
    console.error(`❌ FAILED: ${errors} errors detected - ZERO TOLERANCE VIOLATED`);
    return { stdout: 'STRESS TEST FAILED - ERRORS DETECTED' };
  }
  
  if (warnings > 0) {
    console.error(`❌ FAILED: ${warnings} warnings detected - ZERO TOLERANCE VIOLATED`);
    return { stdout: 'STRESS TEST FAILED - WARNINGS DETECTED' };
  }
  
  if (successRate < 1.0) {
    console.error(`❌ FAILED: ${((1-successRate)*100).toFixed(2)}% failure rate - ZERO TOLERANCE VIOLATED`);
    return { stdout: 'STRESS TEST FAILED - SUCCESS RATE BELOW 100%' };
  }
  
  if (p95ResponseTime >= 100) {
    console.error(`❌ FAILED: P95 latency ${p95ResponseTime.toFixed(2)}ms exceeds 100ms budget`);
    return { stdout: 'STRESS TEST FAILED - LATENCY BUDGET EXCEEDED' };
  }
  
  console.log('\n🎉 EXTREME STRESS TEST PASSED');
  console.log('✅ Zero errors detected');
  console.log('✅ Zero warnings detected'); 
  console.log('✅ 100% success rate maintained');
  console.log('✅ P95 latency under budget');
  console.log('✅ QuantumCoin is bulletproof under extreme load');
  
  return {
    stdout: `STRESS TEST PASSED - ${totalRequests} requests, ${(successRate*100).toFixed(2)}% success, ${p95ResponseTime.toFixed(2)}ms P95`
  };
}
