const express = require("express");
const cors = require("cors");
const bodyParser = require("body-parser");
const wallet = require("./routes/wallet");
const send = require("./routes/send");
const revstop = require("./routes/revstop");

const app = express();
app.use(cors());
app.use(bodyParser.json());

app.use("/wallet", wallet);
app.use("/send", send);
app.use("/revstop", revstop);

app.get("/", (req, res) => res.send("QuantumCoin API is live 🚀"));

const PORT = process.env.PORT || 3001;
app.listen(PORT, () => console.log(`API running on port ${PORT}`));