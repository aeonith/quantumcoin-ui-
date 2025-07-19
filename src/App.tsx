import React from "react";
import WalletInfo from "./components/WalletInfo";
import MineBlock from "./components/MineBlock";
import RevStopToggle from "./components/RevStopToggle";

function App() {
  return (
    <div style={{ padding: "2rem" }}>
      <h1>🚀 QuantumCoin Wallet UI</h1>
      <WalletInfo />
      <hr />
      <MineBlock />
      <hr />
      <RevStopToggle />
    </div>
  );
}

export default App;