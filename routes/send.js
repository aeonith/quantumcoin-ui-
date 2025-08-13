const express = require("express");
const router = express.Router();
const { exec } = require("child_process");

router.post("/", (req, res) => {
  const { from_wallet, to_address, amount } = req.body;
  const cmd = `cargo run --release -- send --from ${from_wallet} --to ${to_address} --amount ${amount}`;
  
  exec(cmd, { cwd: "../rust-core" }, (error, stdout, stderr) => {
    if (error) return res.status(500).json({ error: stderr });
    res.json({ tx: stdout.trim() });
  });
});

module.exports = router;