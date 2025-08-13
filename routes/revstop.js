const express = require("express");
const router = express.Router();
const { exec } = require("child_process");

router.post("/toggle", (req, res) => {
  const { wallet_file, password } = req.body;
  const cmd = `cargo run --release -- revstop_toggle --wallet ${wallet_file} --password ${password}`;
  
  exec(cmd, { cwd: "../rust-core" }, (error, stdout, stderr) => {
    if (error) return res.status(500).json({ error: stderr });
    res.json({ status: stdout.trim() });
  });
});

module.exports = router;