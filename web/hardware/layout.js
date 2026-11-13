import {breadboardContact as contact} from './breadboard.js';
import {analyzerHeader} from './circuit.js';

export const analyzerGeometry={usb:[-27.5,7,0],header:[29,7,0],pinLength:6,pins:analyzerHeader.flatMap((pair,row)=>pair.map((name,column)=>({name,position:[36.5,7+(column===0?1.27:-1.27),(row-2)*2.54]})))};
const analyzerPosition=[45,0,155];
export const analyzerBreakout={signals:['CH0','CH1','CH2','CH3','CH4','CH5','CH6','CH7','CLK','GND'],source:'https://learn.lonelybinary.com/manuals/logic-analyzer/the-kit-in-front-of-you',contacts:['near','far'].flatMap((side,row)=>['CH0','CH1','CH2','CH3','CH4','CH5','CH6','CH7','CLK','GND'].map((name,index)=>({name,side,position:[39.5+index*2.54,13.8,row===0?-5.08:5.08]})))};
export function analyzerPinPosition(name){const pin=analyzerBreakout.contacts.find(pin=>pin.name===name&&pin.side==='near');if(!pin)throw new Error('Unknown analyzer pin: '+name);return pin.position.map((value,axis)=>value+analyzerPosition[axis]);}

const at=id=>contact(id).position;
export const tphGeometry={position:[27.3,7.2,-1.2],pins:Array.from({length:12},(_,i)=>({number:i+1,position:[i<6?-1.27:1.27,6.8,((i%6)-2.5)*2.54]}))};
export function tphPinPosition(number){const pin=tphGeometry.pins.find(pin=>pin.number===number);if(!pin)throw new Error('Unknown TPH2 pin');return pin.position.map((value,axis)=>value+tphGeometry.position[axis]);}
const lead=(from,to,points,kind='jumper',jacket='black',rise=35)=>{const path=[...points];if(/^[A-J][0-9]+$/.test(from))path.splice(1,0,[points[0][0],to.startsWith('TPH2.')?rise:from==='D60'?17:21,points[0][2]]);if(/^[A-J][0-9]+$/.test(to))path.splice(path.length-1,0,[points.at(-1)[0],21,points.at(-1)[2]]);return {from,to,points:path,kind,jacket};};
const probes={
 ch1:{net:'pulse',tip:[-110.24,13,145],tail:[-110.24,3.4,239.5147],ground:[-110.24,11.181,162.918]}
};

export const benchLayout={
  units:'mm',table:{width:900,depth:600,grid:25},
  parts:{
    mac:{name:'Acquisition host · laptop reference',position:[200,0,-150],size:[312.6,15.5,221.2],basis:'Apple published 14-inch M4 Pro/Max enclosure dimensions. Open lid, keyboard, trackpad and connector placement are photo-informed approximations, not Apple CAD.',source:'https://support.apple.com/en-us/121553'},
    windows:{name:'Programming host',position:[-40,0,-270],size:[100,2,24],basis:'Example programming host for JTAG and optional USB acquisition. The host is outside the bench layout; its chassis is not modeled.',source:'https://docs.amd.com/r/2025.2-English/ug973-vivado-release-notes-install-license/Supported-Operating-Systems'},
    scope:{name:'Hantek DSO2D15',position:[-205,0,-180],size:[318,150,110],basis:'Published enclosure dimensions; simplified shell and procedural front. Probe/port locations are photo estimates.',source:'https://www.hantek.com/m/productdetail/17182'},
    breadboard:{name:'830-point breadboard',position:[-95,0,80],size:[165.1,9.6,55.88],basis:'Proposed standard-pitch layout: 63 × 10 terminal holes plus 200 rail holes. Outer dimensions are not measured from the kit.',source:'https://lonelybinary.com/products/electronics-starter-kit-400-pieces'},
    pico:{name:'Pico 2 with headers',position:[-93.73,12.6,80],size:[52.3,3.765,21],basis:'Official Pico 2 STEP; added straight header geometry uses 2.54 mm pitch. Header projection remains unmeasured.',source:'https://pip.raspberrypi.com/categories/1005-raspberry-pi-pico-2'},
    cora:{name:'Cora Z7-07S',position:[-40,0,10],size:[101.6,15,57.9],basis:'Research-reported Cora envelope plus Digilent reference photos; simplified PCB/connectors, not manufacturer CAD.',source:'https://www.mdpi.com/2227-7080/8/2/28'},
    tph:{name:'Pmod TPH2',position:[27.3,7.2,-1.2],size:[33,1.6,20],basis:'Photo-informed illustrative PCB envelope, not a manufacturer dimension; 2.54 mm test-post pitch. Adapter is attached to the modeled JA connector.',source:'https://digilent.com/shop/pmod-tph2-12-pin-test-point-header/'},
    analyzer:{name:'Lonely Binary logic analyzer',position:analyzerPosition,size:[55,14,26],basis:'Estimated enclosure dimensions. Example Mini-USB variant: USB and outward-facing 2×5 signal header on opposite short ends; no connector halfway along the case. Supplier channel-pair map; physical pin orientation still needs checking against the actual device lid. Small breadboard adapter fitted: two duplicated rows 0–7, CLK, GND, with labelled socket endpoints. Adapter dimensions and seating are illustrative; read the actual silkscreen.',source:'https://learn.lonelybinary.com/manuals/logic-analyzer'},
    sensor:{name:'AS5600 module · phase 2',position:[-220,0,105],size:[23,3,23],basis:'Module footprint is an explicit layout estimate. Module pin positions and supply circuit remain unverified.',source:'https://www.infineon.com/assets/row/public/documents/30/49/infineon-as5600-datasheet-en.pdf'}
  },
  probes,
  probeReference:{name:'Hantek passive-probe reference',bodyMm:95,basis:'HT-100B/200B assembly reference: 95 mm probe body, sprung hook, ground collar and coax. Exact supplied probe model, diameters and lead dimensions remain unverified. Routed coax is diagrammatic, not a certified 1.2 m cable model.',source:'https://www.hantek.com/uploadpic/hantek/files/20231121/HT-100B%F3%80%81200BManual.pdf'},
  nets:{
    usbPico:{label:'Pico USB',host:true,color:'--rb-nc-arc',from:'Acquisition-host USB-C data port',to:'Pico Micro-USB · power + console',observe:'No live USB session',points:[[43.7,8,-235],[25,5,-235],[-10,2,-80],[-185,2,-45],[-190,2,80],[-145,14,80],[-120.53,14,80]]},
    usbAnalyzer:{label:'Analyzer USB',host:true,color:'--rb-nc-arc',from:'Acquisition-host USB-C → Mini-USB data cable',to:'Analyzer Mini-USB · power + digital capture',observe:'PulseView · device detection not checked',points:[[43.7,8,-218],[35,5,-218],[15,2,-65],[85,2,-30],[95,2,200],[0,2,200],[0,7,155],analyzerGeometry.usb.map((value,axis)=>value+analyzerPosition[axis])]},
    usbCora:{label:'Cora USB',host:true,color:'--rb-nc-arc',from:'Programming-host USB → Micro-USB-B data cable',to:'Cora J12 · USB-JTAG / UART + power',observe:'Vivado Hardware Manager · not connected',points:[[-65,5,-258],[-70,2,-245],[-100,2,-120],[-62,2,-65],[-58.75,4.1,-32],[-58.75,4.1,-20]]},
    usbScope:{label:'Scope USB',host:true,color:'--rb-nc-arc',from:'Programming-host USB → USB-B data cable',to:'Hantek rear USB device port',observe:'PC capture optional · mains power separate',points:[[-15,5,-258],[-15,2,-265],[-140,2,-265],[-140,50,-235]]},
    pulse:{label:'Pulse',color:'--rb-nc-feed',from:'Pico GP2 · pin 4 · A26',to:'A–E55 → TPH2 pin 1 / Cora JA1',observe:'Scope CH1 · analyzer CH0',runs:[
      lead('A26','B55',[at('A26'),[-104,18,103],[-20,24,110],[-20,24,88],at('B55')],'jumper','yellow'),
      lead('A55','TPH2.1',[at('A55'),[-12,20,65],[21,20,15],[21,26,-24],[26.03,24,-24],tphPinPosition(1)],'jumper','yellow'),
      lead('E55','CH1.tip',[at('E55'),[-48,28,65],[-95,8,130],probes.ch1.tip],'pigtail','yellow'),
      lead('D55','Analyzer.CH0',[at('D55'),[-10,32,105],[60,26,139],[84.5,24,149.92],analyzerPinPosition('CH0')],'jumper','yellow'),
      lead('CH1.coax','Scope.CH1',[probes.ch1.tail,[-155,2,255],[-330,2,230],[-365,4,-85],[-160,42,-113]],'coax')
    ]},
    response:{label:'Response',color:'--rb-nc-axis',from:'Cora JA2 / Y19 · TPH2 pin 2',to:'A–E60 · response strip',observe:'Scope CH2 · analyzer CH1',runs:[
      lead('TPH2.2','A60',[tphPinPosition(2),[26.03,30,-8.81],[40,30,35],[10,18,80],at('A60')],'jumper','blue'),
      lead('D60','Analyzer.CH1',[at('D60'),[2,36,86.35],[45,30,120],[87.04,24,149.92],analyzerPinPosition('CH1')],'jumper','blue'),
    ]},
    ground:{label:'Ground',color:'--rb-color-tertiary',from:'Pico GND · pin 3 · A25',to:'A–E45 → TPH2 GND pin 5',observe:'CH1 probe ground · analyzer GND',runs:[
      lead('A25','B45',[at('A25'),[-110,20,111],[-80,24,111],[-75,24,91],at('B45')]),
      lead('A45','TPH2.5',[at('A45'),[-60,28,45],[-4,28,23],[31.11,20,15],tphPinPosition(5)]),
      lead('E45','CH1.ground',[at('E45'),[-100,40,58],[-150,8,135],probes.ch1.ground],'ground-clip'),
      lead('C45','Analyzer.GND',[at('C45'),[-35,22,100],[55,20,140],[107.36,20,149.92],analyzerPinPosition('GND')],'jumper')
    ]},
    uartTx:{label:'UART TX',ddcs:true,color:'--rb-nc-tool',from:'Pico GP0 · physical pin 1 · A23',to:'TPH2 P3 / JA3 / Y16 · FPGA UART RX',observe:'115200 baud · Pico output → FPGA input',runs:[
      lead('A23','TPH2.3',[at('A23'),[-66,28,43],[26.03,25,1.34],tphPinPosition(3)],'jumper','green',21)
    ]},
    uartRx:{label:'UART RX',ddcs:true,color:'--rb-color-text',from:'TPH2 P4 / JA4 / Y17 · FPGA UART TX',to:'Pico GP1 · physical pin 2 · A24',observe:'115200 baud · FPGA output → Pico input',runs:[
      lead('A24','TPH2.4',[at('A24'),[-64,31,46],[26.03,28,3.88],tphPinPosition(4)],'jumper','white',23)
    ]},
    sda:{label:'SDA · phase 2',color:'--rb-nc-tool',from:'Pico GP4 · physical pin 6',to:'AS5600 SDA · header position unverified',observe:'Analyzer CH2 · 100 kHz I²C',points:[[-105.16,13,93.97],[-135,8,130],[-200,2,130],[-221.27,6,114]],branches:[[[-105.16,13,91.43],[-140,5,220],[105,8.27,220],[96,8.27,152.46],analyzerPinPosition('CH2')]],phase2:true},
    scl:{label:'SCL · phase 2',color:'--rb-nc-arc',from:'Pico GP5 · physical pin 7',to:'AS5600 SCL · header position unverified',observe:'Analyzer CH3 · 100 kHz I²C',points:[[-102.62,13,93.97],[-130,12,137],[-210,2,137],[-218.73,6,114]],branches:[[[-102.62,13,91.43],[-145,5,228],[112,5.73,228],[96,5.73,152.46],analyzerPinPosition('CH3')]],phase2:true}
    ,sensorPower:{label:'3.3 V · phase 2',color:'--rb-color-danger',from:'Pico 3V3 OUT · physical pin 36',to:'AS5600 documented 3.3 V input · module layout pending',observe:'Sensor supply; not joined to Cora VCC',points:[[-107.7,13,66.03],[-130,16,55],[-250,2,55],[-226.35,6,114]],phase2:true},
    sensorGround:{label:'Sensor GND · phase 2',color:'--rb-color-tertiary',from:'TPH2 GND · pin 11',to:'AS5600 GND · module layout pending',observe:'Shared 0 V reference',points:[tphPinPosition(11),[60,5,245],[-250,5,245],[-223.81,6,114]],phase2:true}
  }
};

export function benchLayoutFor(profile='pulse'){
  if(!['pulse','ddcs'].includes(profile))throw new Error('Unknown bench profile');
  const layout=structuredClone(benchLayout);layout.profile=profile;
  if(profile==='ddcs'){
    Object.assign(layout.nets.pulse,{label:'STEP',from:'FPGA JA1 / Y18 · TPH2 P1 output',to:'Row 55 · analyzer CH0 / scope CH1 · Pico GP2 input',observe:'Finite DDCS job · GP2 remains input / high impedance'});
    Object.assign(layout.nets.response,{label:'DIR',from:'FPGA JA2 / Y19 · TPH2 P2 output',to:'Row 60 · analyzer CH1',observe:'Direction sampled at each STEP rising edge'});
  }
  return layout;
}

export function benchNetVisible(net,{hosts=false,phase2=false,profile='pulse'}={}){
  return hosts?!!net.host:phase2?!!net.phase2:!net.host&&!net.phase2&&(!net.ddcs||profile==='ddcs');
}

export function benchGridLines(){const points=[],{width,depth,grid}=benchLayout.table;for(let x=-width/2;x<=width/2;x+=grid)points.push(x,.06,-depth/2,x,.06,depth/2);for(let z=-depth/2;z<=depth/2;z+=grid)points.push(-width/2,.06,z,width/2,.06,z);return points;}

export function laptopKeys(){
  const keys=[],rows=[[1.2,...Array(12).fill(1),1.2],Array(14).fill(1),[1.5,...Array(12).fill(1),1.5],[1.7,...Array(11).fill(1),1.7],[2.1,...Array(10).fill(1),2.1]];
  function row(widths,z,span=256){const unit=span/widths.reduce((a,b)=>a+b,0);let x=-128;for(const width of widths){keys.push({x:x+width*unit/2,z,width:width*unit-2,depth:13});x+=width*unit;}}
  rows.forEach((widths,i)=>row(widths,-80+i*17));row([1,1,1,1.25,5,1.25,1],5,204);
  for(const [x,z,depth] of [[86,5,13],[103,1,6],[103,9,6],[120,5,13]])keys.push({x,z,width:14,depth});return keys;
}
