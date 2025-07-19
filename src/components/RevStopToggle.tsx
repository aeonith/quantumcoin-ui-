import React, { useState } from "react";
import { toggleRevStop } from "../api";

const RevStopToggle = () => {
  const [status, setStatus] = useState("");

  const handleEnable = async () => {
    const result = await toggleRevStop(true);
    setStatus(result);
  };

  const handleDisable = async () => {
    const result = await toggleRevStop(false);
    setStatus(result);
  };

  return (
    <div>
      <h3>🔒 RevStop</h3>
      <button onClick={handleEnable}>Enable</button>
      <button onClick={handleDisable}>Disable</button>
      <p>{status}</p>
    </div>
  );
};

export default RevStopToggle;