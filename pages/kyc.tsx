import { useEffect } from "react";

export default function KYC() {
  useEffect(() => {
    // Redirect to legacy KYC page for now
    window.location.href = "/kyc.html";
  }, []);

  return (
    <div className="min-h-screen bg-quantum-dark flex items-center justify-center">
      <div className="text-cyan-300">Redirecting to KYC verification...</div>
    </div>
  );
}
