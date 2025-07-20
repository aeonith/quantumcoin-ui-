const express = require('express');
const cors = require('cors');
const app = express();
const PORT = process.env.PORT || 3001;

app.use(cors());

app.get('/api/price', async (req, res) => {
  try {
    // Replace with your actual logic or price API
    const mockPrice = 0.035722; // Example dynamic price
    res.json({ price: mockPrice });
  } catch (err) {
    res.status(500).json({ error: 'Failed to fetch price' });
  }
});

app.listen(PORT, () => {
  console.log(`QuantumCoin backend listening on port ${PORT}`);
});