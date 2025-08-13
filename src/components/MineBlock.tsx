import React, { useState } from "react";
import { mineBlock } from "../api";

const MineBlock = () => {
  const [status, setStatus] = useState("");

  const handleMine = async () => {
    setStatus("⛏ Mining in progress...");
    const result = await mineBlock();
    setStatus(result);
  };

  return (
    <div>
      <h3>⛏ Mine New Block</h3>
      <button onClick={handleMine}>Mine</button>
      <p>{status}</p>
    </div>
  );
};

export default MineBlock;