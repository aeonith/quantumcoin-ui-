import React from 'react';
import { NavigationContainer } from '@react-navigation/native';
import { createNativeStackNavigator } from '@react-navigation/native-stack';
import HomeScreen from './screens/HomeScreen';
import SendScreen from './screens/SendScreen';
import BuyScreen from './screens/BuyScreen';

const Stack = createNativeStackNavigator();

export default function RootNav() {
  return (
    <NavigationContainer>
      <Stack.Navigator 
        screenOptions={{
          headerStyle: { backgroundColor: '#101521' },
          headerTintColor: '#fff'
        }}
      >
        <Stack.Screen 
          name="Home" 
          component={HomeScreen} 
          options={{ title: 'QuantumCoin' }}
        />
        <Stack.Screen name="Send" component={SendScreen} />
        <Stack.Screen name="Buy" component={BuyScreen} />
      </Stack.Navigator>
    </NavigationContainer>
  );
}
