export const benchPower=[
  {id:'pico',device:'Pico 2',from:'Acquisition host',cable:'USB-C → Micro-USB',supply:'USB 5 V → onboard 3.3 V regulator',role:'Power + MicroPython console'},
  {id:'analyzer',device:'Logic analyzer',from:'Acquisition host',cable:'USB-C → Mini-USB data cable',supply:'USB powered; signal pins are inputs',role:'Power + PulseView capture'},
  {id:'cora',device:'Cora Z7-07S',from:'Programming host',cable:'USB → J12 Micro-USB',supply:'JP3 selects USB power; onboard regulators',role:'Power + Vivado JTAG / UART'},
  {id:'scope',device:'Hantek DSO2D15',from:'Grounded mains outlet',cable:'Scope mains lead',supply:'Internal supply; USB is a data link',role:'CH1 pulse · CH2 unused; analyzer CH1 return'},
  {id:'breadboard',device:'Breadboard',from:'No supply connected',cable:'Signal and ground jumpers only',supply:'Power rails unused; kit module disconnected',role:'Passive five-hole signal strips'}
];
