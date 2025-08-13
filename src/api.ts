// src/api.ts
const BASE_URL = "https://quantumcoin-ithu.onrender.com";

export const fetchAddress = async (): Promise<string> => {
  const res = await fetch(`${BASE_URL}/address`);
  return res.text();
};

export const fetchBalance = async (): Promise<string> => {
  const res = await fetch(`${BASE_URL}/balance`);
  return res.text();
};

export const mineBlock = async (): Promise<string> => {
  const res = await fetch(`${BASE_URL}/mine`, { method: "POST" });
  return res.text();
};

export const toggleRevStop = async (enable: boolean): Promise<string> => {
  const res = await fetch(`${BASE_URL}/revstop`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ enable }),
  });
  return res.text();
};