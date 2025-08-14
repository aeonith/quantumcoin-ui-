import React, {createContext, useContext, useEffect, useState} from "react";

type User = { id: string; email: string } | null;
type Ctx = {
  user: User;
  login: (email: string, password: string) => Promise<void>;
  register: (email: string, password: string) => Promise<void>;
  logout: () => void;
};
const Context = createContext<Ctx | undefined>(undefined);

export const AuthProvider: React.FC<{children: React.ReactNode}> = ({children}) => {
  const [user, setUser] = useState<User>(null);
  useEffect(() => { const raw = localStorage.getItem("qc_user"); if (raw) setUser(JSON.parse(raw)); }, []);
  const save = (u: User) => { u ? localStorage.setItem("qc_user", JSON.stringify(u)) : localStorage.removeItem("qc_user"); setUser(u); };
  const login = async (email: string) => save({id:"local", email});
  const register = async (email: string) => save({id:"local", email});
  const logout = () => save(null);
  return <Context.Provider value={{user, login, register, logout}}>{children}</Context.Provider>;
};
export const useAuth = () => { const v = useContext(Context); if(!v) throw new Error("useAuth outside provider"); return v; };
