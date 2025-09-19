import * as SecureStore from 'expo-secure-store';
import nacl from 'tweetnacl';
import bs58 from 'bs58';

const SEED_KEY = 'qtc.seed';

export type Wallet = { 
  publicKey: string; 
  secretKey: string; 
};

export async function getOrCreateWallet(): Promise<Wallet> {
  let seed = await SecureStore.getItemAsync(SEED_KEY);
  
  if (!seed) {
    const kp = nacl.sign.keyPair();
    await SecureStore.setItemAsync(SEED_KEY, bs58.encode(kp.secretKey));
    return { 
      publicKey: bs58.encode(kp.publicKey), 
      secretKey: bs58.encode(kp.secretKey) 
    };
  }
  
  const sk = bs58.decode(seed);
  const kp = nacl.sign.keyPair.fromSecretKey(sk);
  return { 
    publicKey: bs58.encode(kp.publicKey), 
    secretKey: bs58.encode(kp.secretKey) 
  };
}

export function signMessage(secretKeyB58: string, message: string): string {
  const secretKey = bs58.decode(secretKeyB58);
  const sig = nacl.sign.detached(new TextEncoder().encode(message), secretKey);
  return bs58.encode(sig);
}
