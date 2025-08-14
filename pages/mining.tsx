import { useEffect } from "react";
import { useRouter } from "next/router";

export default function Mining() {
  const router = useRouter();

  useEffect(() => {
    // Redirect to legacy mining page for now
    window.location.href = "/mining.html";
  }, []);

  return (
    <div className="min-h-screen bg-quantum-dark flex items-center justify-center">
      <div className="text-cyan-300">Redirecting to mining interface...</div>
    </div>
  );
}
