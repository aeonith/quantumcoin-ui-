const express = require("express");
const router = express.Router();
const { exec } = require("child_process");

router.post("/balance", (req, res) => {
  const { wallet_file } = req.body;
  const cmd = `cargo run --release -- get_balance --wallet ${wallet_file}`;
  
  exec(cmd, { cwd: "../rust-core" }, (error, stdout, stderr) => {
    if (error) return res.status(500).json({ error: stderr });
    res.json({ balance: stdout.trim() });
  });
});

module.exports = router;