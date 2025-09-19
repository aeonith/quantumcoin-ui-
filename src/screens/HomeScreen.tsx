import React, { useEffect, useState } from 'react';
import { View, Text, Button } from 'react-native';
import RainBackground from '../components/RainBackground';
import { login } from '../api/auth';
import { fetchMe } from '../api/wallet';

export default function HomeScreen({ navigation }: any) {
  const [address, setAddress] = useState<string>();
  const [bal, setBal] = useState('0');

  useEffect(() => { 
    (async () => {
      const { address } = await login(); 
      setAddress(address);
      const me = await fetchMe(); 
      setBal(me.balances?.QTC ?? '0');
    })(); 
  }, []);

  return (
    <View style={{
      flex: 1,
      backgroundColor: '#0B0F17',
      padding: 16,
      justifyContent: 'center'
    }}>
      <RainBackground />
      
      <Text style={{ color: '#fff' }}>Address:</Text>
      <Text 
        selectable 
        style={{ 
          color: '#8BC6FF', 
          marginBottom: 16 
        }}
      >
        {address ?? '...'}
      </Text>
      
      <Text style={{ color: '#fff' }}>QTC Balance:</Text>
      <Text style={{ 
        color: '#B388FF', 
        fontSize: 28, 
        fontWeight: '700', 
        marginBottom: 24 
      }}>
        {bal}
      </Text>
      
      <Button 
        title="Send" 
        onPress={() => navigation.navigate('Send')} 
      />
      
      <View style={{ height: 12 }} />
      
      <Button 
        title="Buy QTC" 
        onPress={() => navigation.navigate('Buy')} 
      />
    </View>
  );
}
