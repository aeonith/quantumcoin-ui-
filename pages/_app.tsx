import type { AppProps } from "next/app";
import { AuthProvider } from "@/context/AuthContext";
import { RevStopProvider } from "@/context/RevStopContext";
import "@/styles/globals.css";

export default function App({ Component, pageProps }: AppProps) {
  return (
    <AuthProvider>
      <RevStopProvider>
        <Component {...pageProps} />
      </RevStopProvider>
    </AuthProvider>
  );
}
