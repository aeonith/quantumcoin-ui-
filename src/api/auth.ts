import { api, setAuthToken } from './client';
import { getOrCreateWallet, signMessage } from '../crypto/wallet';

export async function login(): Promise<{ jwt: string; address: string }> {
  const w = await getOrCreateWallet();
  const address = w.publicKey;
  
  const { data: { nonce } } = await api.get<{nonce:string}>('/auth/nonce', { 
    params: { address } 
  });
  
  const signature = signMessage(w.secretKey, nonce);
  
  const { data: { jwt } } = await api.post<{jwt:string}>('/auth/verify', { 
    address, 
    signature, 
    nonce 
  });
  
  setAuthToken(jwt);
  return { jwt, address };
}
