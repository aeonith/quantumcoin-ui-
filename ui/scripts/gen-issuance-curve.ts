#!/usr/bin/env tsx
/**
 * Generate QuantumCoin issuance curve as SVG
 * 
 * This script generates a visual representation of the QuantumCoin supply
 * issuance schedule showing the halving curve over 66 years.
 */

import { writeFileSync } from 'fs';
import { join } from 'path';
import { ECONOMICS, CALCULATED, IssuanceCalculator } from '../src/lib/economics';

interface DataPoint {
  year: number;
  height: number;
  cumulativeSupply: number;
  blockReward: number;
}

function generateIssuanceCurve(): DataPoint[] {
  const dataPoints: DataPoint[] = [];
  const maxYears = ECONOMICS.HALVING_DURATION_YEARS;
  const pointsPerYear = 4; // Quarterly data points
  
  for (let year = 0; year <= maxYears; year += 1 / pointsPerYear) {
    const height = Math.floor(year * CALCULATED.BLOCKS_PER_YEAR);
    const cumulativeSupply = IssuanceCalculator.getCumulativeIssuance(height);
    const blockReward = IssuanceCalculator.getBlockReward(height);
    
    dataPoints.push({
      year,
      height,
      cumulativeSupply,
      blockReward,
    });
  }
  
  return dataPoints;
}

function generateSVG(data: DataPoint[]): string {
  const width = 800;
  const height = 400;
  const margin = { top: 40, right: 80, bottom: 60, left: 80 };
  const chartWidth = width - margin.left - margin.right;
  const chartHeight = height - margin.top - margin.bottom;
  
  // Scales
  const maxYear = Math.max(...data.map(d => d.year));
  const maxSupply = ECONOMICS.TOTAL_SUPPLY;
  
  const xScale = (year: number) => (year / maxYear) * chartWidth;
  const yScale = (supply: number) => chartHeight - (supply / maxSupply) * chartHeight;
  
  // Generate path for supply curve
  const pathData = data.map((d, i) => {
    const x = xScale(d.year);
    const y = yScale(d.cumulativeSupply);
    return `${i === 0 ? 'M' : 'L'} ${x + margin.left} ${y + margin.top}`;
  }).join(' ');
  
  // Generate halving markers
  const halvingMarkers = [];
  for (let period = 1; period < CALCULATED.TOTAL_HALVINGS; period++) {
    const year = period * ECONOMICS.HALVING_PERIOD_YEARS;
    const x = xScale(year) + margin.left;
    
    halvingMarkers.push(`
      <line x1="${x}" y1="${margin.top}" x2="${x}" y2="${height - margin.bottom}" 
            stroke="#ff6b6b" stroke-width="1" stroke-dasharray="5,5" opacity="0.6"/>
      <text x="${x}" y="${margin.top - 5}" text-anchor="middle" font-size="10" fill="#666">
        H${period}
      </text>
    `);
  }
  
  // X-axis ticks
  const xTicks = [];
  for (let year = 0; year <= maxYear; year += 10) {
    const x = xScale(year) + margin.left;
    xTicks.push(`
      <line x1="${x}" y1="${height - margin.bottom}" x2="${x}" y2="${height - margin.bottom + 5}" 
            stroke="#999" stroke-width="1"/>
      <text x="${x}" y="${height - margin.bottom + 20}" text-anchor="middle" font-size="12" fill="#333">
        ${year}
      </text>
    `);
  }
  
  // Y-axis ticks
  const yTicks = [];
  for (let supply = 0; supply <= maxSupply; supply += 5000000) {
    const y = yScale(supply) + margin.top;
    yTicks.push(`
      <line x1="${margin.left - 5}" y1="${y}" x2="${margin.left}" y2="${y}" 
            stroke="#999" stroke-width="1"/>
      <text x="${margin.left - 10}" y="${y + 4}" text-anchor="end" font-size="12" fill="#333">
        ${supply / 1000000}M
      </text>
    `);
  }
  
  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
  <!-- Background -->
  <rect width="${width}" height="${height}" fill="#fafafa"/>
  
  <!-- Title -->
  <text x="${width / 2}" y="25" text-anchor="middle" font-size="18" font-weight="bold" fill="#333">
    QuantumCoin Supply Issuance Schedule
  </text>
  
  <!-- Chart area -->
  <rect x="${margin.left}" y="${margin.top}" width="${chartWidth}" height="${chartHeight}" 
        fill="white" stroke="#ddd" stroke-width="1"/>
  
  <!-- Grid lines -->
  ${yTicks.map(tick => tick.replace('stroke="#999"', 'stroke="#f0f0f0"').replace('x2="${margin.left}"', `x2="${width - margin.right}"`)).join('')}
  
  <!-- Supply curve -->
  <path d="${pathData}" stroke="#4a90e2" stroke-width="3" fill="none"/>
  
  <!-- Fill under curve -->
  <path d="${pathData} L ${xScale(maxYear) + margin.left} ${yScale(0) + margin.top} L ${margin.left} ${yScale(0) + margin.top} Z" 
        fill="url(#gradient)" opacity="0.3"/>
  
  <!-- Gradient definition -->
  <defs>
    <linearGradient id="gradient" x1="0%" y1="0%" x2="0%" y2="100%">
      <stop offset="0%" style="stop-color:#4a90e2;stop-opacity:1" />
      <stop offset="100%" style="stop-color:#4a90e2;stop-opacity:0" />
    </linearGradient>
  </defs>
  
  <!-- Halving markers -->
  ${halvingMarkers.join('')}
  
  <!-- Axes -->
  <line x1="${margin.left}" y1="${height - margin.bottom}" x2="${width - margin.right}" y2="${height - margin.bottom}" 
        stroke="#333" stroke-width="2"/>
  <line x1="${margin.left}" y1="${margin.top}" x2="${margin.left}" y2="${height - margin.bottom}" 
        stroke="#333" stroke-width="2"/>
  
  <!-- Axis ticks -->
  ${xTicks.join('')}
  ${yTicks.join('')}
  
  <!-- Axis labels -->
  <text x="${width / 2}" y="${height - 10}" text-anchor="middle" font-size="14" fill="#333">
    Years
  </text>
  <text x="20" y="${height / 2}" text-anchor="middle" font-size="14" fill="#333" transform="rotate(-90, 20, ${height / 2})">
    Cumulative Supply (QTC)
  </text>
  
  <!-- Key statistics -->
  <g transform="translate(${width - margin.right + 10}, ${margin.top + 20})">
    <rect x="0" y="0" width="70" height="80" fill="white" stroke="#ddd" stroke-width="1" rx="5"/>
    <text x="5" y="15" font-size="10" font-weight="bold" fill="#333">Key Stats</text>
    <text x="5" y="30" font-size="9" fill="#666">Total: ${ECONOMICS.TOTAL_SUPPLY.toLocaleString()}</text>
    <text x="5" y="42" font-size="9" fill="#666">Halvings: ${CALCULATED.TOTAL_HALVINGS}</text>
    <text x="5" y="54" font-size="9" fill="#666">Period: ${ECONOMICS.HALVING_PERIOD_YEARS}y</text>
    <text x="5" y="66" font-size="9" fill="#666">Block: ${ECONOMICS.BLOCK_TIME_TARGET_SEC}s</text>
  </g>
  
  <!-- Max supply line -->
  <line x1="${margin.left}" y1="${yScale(maxSupply) + margin.top}" 
        x2="${width - margin.right}" y2="${yScale(maxSupply) + margin.top}" 
        stroke="#e74c3c" stroke-width="2" stroke-dasharray="10,5"/>
  <text x="${width - margin.right - 5}" y="${yScale(maxSupply) + margin.top - 5}" 
        text-anchor="end" font-size="10" fill="#e74c3c" font-weight="bold">
    Max Supply: ${maxSupply.toLocaleString()} QTC
  </text>
</svg>`;
}

// Generate and save the curve
console.log('📈 Generating QuantumCoin issuance curve...');

const data = generateIssuanceCurve();
const svg = generateSVG(data);

// Save to public directory
const outputPath = join(process.cwd(), 'public', 'issuance-curve.svg');
writeFileSync(outputPath, svg, 'utf8');

console.log(`✅ Issuance curve saved to: ${outputPath}`);

// Output some key statistics
const finalData = data[data.length - 1];
console.log('\n📊 Key Statistics:');
console.log(`Total Supply: ${ECONOMICS.TOTAL_SUPPLY.toLocaleString()} QTC`);
console.log(`Final Issued: ${finalData?.cumulativeSupply.toLocaleString()} QTC`);
console.log(`Unutilized: ${(ECONOMICS.TOTAL_SUPPLY - (finalData?.cumulativeSupply ?? 0)).toLocaleString()} QTC`);
console.log(`Halvings: ${CALCULATED.TOTAL_HALVINGS} over ${ECONOMICS.HALVING_DURATION_YEARS} years`);
console.log(`Block Time: ${ECONOMICS.BLOCK_TIME_TARGET_SEC}s (${ECONOMICS.BLOCK_TIME_TARGET_SEC / 60} minutes)`);
