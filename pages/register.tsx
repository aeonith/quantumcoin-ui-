import NavBar from "@/components/NavBar";
import { useAuth } from "@/context/AuthContext";
import { useState } from "react";
import { useRouter } from "next/router";

export default function Register(){
  const { register } = useAuth();
  const [email,setEmail]=useState(""); 
  const [password,setPassword]=useState("");
  const [loading, setLoading] = useState(false);
  const router = useRouter();

  const handleRegister = async () => {
    if (!email) return;
    setLoading(true);
    try {
      await register(email, password);
      router.push("/wallet");
    } catch (error) {
      alert("Registration failed");
    }
    setLoading(false);
  };

  return (
    <main className="min-h-screen bg-[#061018] text-cyan-100">
      <NavBar/>
      <div className="mx-auto max-w-md px-4 py-10">
        <h2 className="text-2xl font-semibold text-cyan-300 mb-4">Create Account</h2>
        <input 
          className="w-full mb-3 p-3 rounded bg-[#0b1b26] border border-cyan-700/30" 
          placeholder="Email" 
          value={email} 
          onChange={e=>setEmail(e.target.value)}
        />
        <input 
          className="w-full mb-4 p-3 rounded bg-[#0b1b26] border border-cyan-700/30" 
          type="password" 
          placeholder="Password" 
          value={password} 
          onChange={e=>setPassword(e.target.value)}
        />
        <button 
          onClick={handleRegister} 
          disabled={loading}
          className="w-full py-3 rounded bg-cyan-500 text-black disabled:opacity-50"
        >
          {loading ? "Creating..." : "Create Account"}
        </button>
      </div>
    </main>
  );
}
