import NavBar from "@/components/NavBar";
import { useAuth } from "@/context/AuthContext";
import { useState, useEffect } from "react";
import { useRouter } from "next/router";
import Link from "next/link";

export default function Login() {
  const { login, user, isLoading } = useAuth();
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isSubmitting, setIsSubmitting] = useState(false);
  const router = useRouter();

  // Redirect if already logged in
  useEffect(() => {
    if (!isLoading && user) {
      router.push("/dashboard");
    }
  }, [user, isLoading, router]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsSubmitting(true);

    try {
      await login(email, password);
      router.push("/dashboard");
    } catch (err: any) {
      setError(err.message || "Login failed");
    } finally {
      setIsSubmitting(false);
    }
  };

  if (isLoading) {
    return (
      <div className="min-h-screen bg-quantum-dark flex items-center justify-center">
        <div className="text-cyan-300">Loading...</div>
      </div>
    );
  }

  if (user) {
    return null; // Will redirect
  }

  return (
    <div className="min-h-screen bg-quantum-dark text-cyan-100">
      <NavBar />
      
      <main className="pt-20 pb-16">
        <div className="mx-auto max-w-md px-4 py-12">
          <div className="quantum-card">
            <h2 className="text-2xl font-semibold text-cyan-300 mb-6 text-center">
              🔐 Login to QuantumCoin™
            </h2>

            <form onSubmit={handleSubmit} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-cyan-300 mb-2">
                  Email Address
                </label>
                <input
                  type="email"
                  className="quantum-input"
                  placeholder="Enter your email"
                  value={email}
                  onChange={(e) => setEmail(e.target.value)}
                  required
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-cyan-300 mb-2">
                  Password
                </label>
                <input
                  type="password"
                  className="quantum-input"
                  placeholder="Enter your password"
                  value={password}
                  onChange={(e) => setPassword(e.target.value)}
                  required
                />
              </div>

              {error && (
                <div className="p-3 bg-red-900/30 border border-red-600/50 rounded text-red-300 text-sm">
                  {error}
                </div>
              )}

              <button
                type="submit"
                disabled={isSubmitting}
                className={`w-full py-3 rounded font-semibold transition-colors ${
                  isSubmitting 
                    ? 'bg-gray-600 text-gray-300 cursor-not-allowed' 
                    : 'quantum-button-primary'
                }`}
              >
                {isSubmitting ? "Logging in..." : "Login"}
              </button>
            </form>

            <div className="mt-6 text-center">
              <p className="text-sm opacity-75">
                Don't have an account?{" "}
                <Link href="/register" className="text-cyan-300 hover:text-cyan-200 underline">
                  Create one here
                </Link>
              </p>
            </div>

            <div className="mt-6 p-3 bg-cyan-900/20 border border-cyan-600/30 rounded-lg">
              <div className="text-cyan-300 text-sm font-medium mb-1">🛡️ Security Notice</div>
              <div className="text-xs opacity-80">
                Your account data is stored locally. For enhanced security, enable RevStop™ 
                after creating your wallet to protect against unauthorized transactions.
              </div>
            </div>
          </div>
        </div>
      </main>
    </div>
  );
}
