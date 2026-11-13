import {CatmullRomCurve3,Vector3} from 'three';

class SupportedWire extends CatmullRomCurve3{
  getPoint(t,target){const point=super.getPoint(t,target);point.y=Math.max(1.05,point.y);return point;}
}

export function supportedWireCurve(points){return new SupportedWire(points.map(point=>new Vector3(...point)),false,'centripetal');}

export const jacketColors={black:'#24272b',yellow:'#d9b526',blue:'#397fc9',green:'#388e64',red:'#bf443b',white:'#e2e1da'};
export const starterKitWires={source:'https://lonelybinary.com/products/electronics-starter-kit-400-pieces',available:{mm:40,mf:40,ff:40}};
export function wireHardware(run){
  const terminal=id=>/^[A-J][0-9]+$/.test(id||'')?'breadboard-male':id?.startsWith('TPH2.')?'female':id?.startsWith('Analyzer.')?'female':id?.endsWith('.tip')?'male-test-pin':id?.endsWith('.ground')?'probe-ground-collar':id?.endsWith('.coax')?'probe-strain-relief':id?.startsWith('Scope.')?'bnc':null;
  if(run.kind==='jumper')return {ends:[terminal(run.from),terminal(run.to)],parts:[[run.from,run.to].some(id=>(id?.startsWith('TPH2.')||id?.startsWith('Analyzer.')))?'mf':'mm']};
  if(run.kind==='pigtail')return {ends:['breadboard-male','male-test-pin'],parts:['mm']};
  if(run.kind==='hook-lead'||run.kind==='ground-clip')return {ends:[terminal(run.from),terminal(run.to)],parts:run.kind==='hook-lead'?['mm','ff','hook']:['mm','probe-ground-lead'],joint:{t:(run.points.length-2)/(run.points.length-1),kind:run.kind==='hook-lead'?'hook':'ground-clip'}};
  if(run.kind==='coax')return {ends:['probe-strain-relief','bnc'],parts:['probe-coax']};
  return {ends:[null,null],parts:[]};
}
export function phase1WireUse(layout){const used={mm:0,mf:0,ff:0,hook:0};for(const id of ['pulse','response','ground',...(layout.profile==='ddcs'?['uartTx','uartRx']:[])])for(const run of wireRuns(layout.nets[id]))for(const part of wireHardware(run).parts)if(part in used)used[part]++;return used;}
export function wireRuns(net){
  if(net.runs)return net.runs.map((run,index)=>({...run,index,jacket:run.kind==='coax'?'black':net.label.startsWith('Pulse')?'yellow':net.label.startsWith('Response')?'blue':net.label.startsWith('Ground')?'black':run.jacket}));
  const jacket=net.host||net.label.startsWith('Ground')||net.label.startsWith('Sensor GND')?'black':net.label.startsWith('Pulse')?'yellow':net.label.startsWith('Response')?'blue':net.label.startsWith('3.3')?'red':net.label.startsWith('SDA')?'green':'yellow';
  const paths=[net.points,...net.branches||[]];return paths.map((points,index)=>({points,index,from:net.from,to:net.to,kind:net.host?'usb':index===2&&['Pulse','Response'].includes(net.label)?'coax':index===1?'hook-lead':'jumper',jacket:index>=2?'black':jacket}));
}

export function endpointLabel(id,profile='pulse'){
  if(!id)return 'Connection not identified';
  if(/^[A-J]\d+$/.test(id))return 'Breadboard hole '+id;
  if(id.startsWith('TPH2.')){const number=Number(id.slice(5));if(profile==='ddcs'&&number<=4)return ['FPGA STEP output · TPH2 P1 / JA1 / Y18','FPGA DIR output · TPH2 P2 / JA2 / Y19','FPGA UART RX input · TPH2 P3 / JA3 / Y16','FPGA UART TX output · TPH2 P4 / JA4 / Y17'][number-1];return number===1?'Cora pulse input · TPH2 P1, pin 1':number===2?'Cora return output · TPH2 P2, pin 2':number===5||number===11?'Cora common ground · TPH2 GND, pin '+number:'Cora · TPH2 pin '+number;}
  if(id.startsWith('Analyzer.'))return 'Analyzer adapter · '+id.slice(9).replace('CH','channel ');
  return {'CH1.tip':'Scope CH1 · normal probe hook tip','CH1.ground':'Scope CH1 · short ground clip','CH1.coax':'Normal probe · fixed coax cable','Scope.CH1':'Scope CH1 · BNC input'}[id]||id;
}

export function wireContext(id,run){
  const hosts={usbPico:['mac','pico','breadboard'],usbAnalyzer:['mac','analyzer'],usbCora:['windows','cora'],usbScope:['windows','scope']};
  if(hosts[id])return hosts[id];
  if(['sda','scl','sensorPower','sensorGround'].includes(id))return ['pico','breadboard','sensor',...(run===1?['analyzer']:[])];
  const contexts={pulse:[[],['cora','tph'],['scope'],['analyzer'],['scope']],response:[['cora','tph'],['analyzer']],ground:[[],['cora','tph'],['scope'],['analyzer']]};
  return ['pico','breadboard',...(run===null?['cora','tph','scope','analyzer']:contexts[id]?.[run]||['cora','tph'])];
}
export function connectionBox(layout,id,run){
  const points=[];
  if(run!==-1)for(const wire of wireRuns(layout.nets[id]).filter(item=>run===null||item.index===run))for(let i=0;i<=100;i++)points.push(supportedWireCurve(wire.points).getPoint(i/100).toArray());
  for(const name of wireContext(id,run)){const part=layout.parts[name],[x,y,z]=part.position,[w,h,d]=part.size;points.push([x-w/2,y,z-d/2],[x+w/2,y+h,z+d/2]);if(name==='mac')points.push([x,260,z-165]);}
  return {min:[0,1,2].map(axis=>Math.min(...points.map(point=>point[axis]))-[12,20,8][axis]),max:[0,1,2].map(axis=>Math.max(...points.map(point=>point[axis]))+[12,20,8][axis])};
}

export function connectionFocusBox(layout,id,run){
  const net=layout.nets[id],runs=wireRuns(net).filter(item=>run===null||item.index===run);
  if(net.host||runs.length===1&&runs[0].kind==='coax')return connectionBox(layout,id,run);
  const points=runs.filter(item=>item.kind!=='coax').flatMap(wire=>wire.points);
  for(const name of wireContext(id,run).filter(name=>!['mac','windows','scope'].includes(name))){const part=layout.parts[name],[x,y,z]=part.position,[w,h,d]=part.size;points.push([x-w/2,y,z-d/2],[x+w/2,y+h,z+d/2]);}
  return {min:[0,1,2].map(axis=>Math.min(...points.map(point=>point[axis]))-[16,20,16][axis]),max:[0,1,2].map(axis=>Math.max(...points.map(point=>point[axis]))+[16,20,16][axis])};
}
