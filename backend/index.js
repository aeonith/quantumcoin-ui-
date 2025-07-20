// backend/index.js
const express = require('express');
const cors = require('cors');
const app = express();
const PORT = process.env.PORT || 3000;

// Allow requests from your frontend
app.use(cors());

// Sample dynamic price API (floor = $0.25)
app.get('/api/price', (req, res) => {
  res.json({ price: 0.25 });
});

app.get('/', (req, res) => {
  res.send('QuantumCoin backend is live!');
});

app.listen(PORT, () => {
  console.log(`🚀 QuantumCoin backend running on port ${PORT}`);
});