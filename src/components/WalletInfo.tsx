import React, { useEffect, useState } from "react";
import { fetchAddress, fetchBalance } from "../api";

const WalletInfo = () => {
  const [address, setAddress] = useState("");
  const [balance, setBalance] = useState("");

  useEffect(() => {
    fetchAddress().then(setAddress);
    fetchBalance().then(setBalance);
  }, []);

  return (
    <div>
      <h2>🔐 Wallet Address</h2>
      <p>{address}</p>
      <h3>💰 Balance</h3>
      <p>{balance} QTC</p>
    </div>
  );
};

export default WalletInfo;