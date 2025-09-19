import { api } from './client';

export type Me = { 
  address: string; 
  balances: Record<string, string>; 
};

export async function fetchMe(): Promise<Me> {
  const { data } = await api.get<Me>('/me');
  return data;
}

export async function sendQtc(to: string, amount: string) {
  const { data } = await api.post<{txId: string}>('/transfer', { to, amount });
  return data;
}

export async function startBuy(amountFiat: number) {
  const { data } = await api.post<{checkoutUrl: string; orderId: string}>('/buy/intent', { 
    amountFiat, 
    onramp: 'moonpay' 
  });
  return data;
}
