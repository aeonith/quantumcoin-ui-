import React, { useState } from 'react';
import { View, TextInput, Button, Text, Alert } from 'react-native';
import { sendQtc } from '../api/wallet';

export default function SendScreen() {
  const [to, setTo] = useState('');
  const [amt, setAmt] = useState('');

  const onSend = async () => { 
    try { 
      const { txId } = await sendQtc(to.trim(), amt.trim()); 
      Alert.alert('Sent', `txId: ${txId}`); 
    } catch(e: any) { 
      Alert.alert('Error', e?.response?.data?.error ?? 'Failed'); 
    } 
  };

  return (
    <View style={{
      flex: 1,
      backgroundColor: '#0B0F17',
      padding: 16
    }}>
      <Text style={{ 
        color: '#fff', 
        marginBottom: 6 
      }}>
        To
      </Text>
      
      <TextInput 
        value={to} 
        onChangeText={setTo} 
        placeholder="QTC..." 
        placeholderTextColor="#666"
        style={{
          backgroundColor: '#101521',
          color: '#fff',
          padding: 12,
          borderRadius: 12,
          marginBottom: 12
        }}
      />
      
      <Text style={{ 
        color: '#fff', 
        marginBottom: 6 
      }}>
        Amount
      </Text>
      
      <TextInput 
        value={amt} 
        onChangeText={setAmt} 
        keyboardType="decimal-pad" 
        placeholder="0.0" 
        placeholderTextColor="#666"
        style={{
          backgroundColor: '#101521',
          color: '#fff',
          padding: 12,
          borderRadius: 12,
          marginBottom: 16
        }}
      />
      
      <Button title="Send" onPress={onSend} />
    </View>
  );
}
