import {picoPins,breadboardConnection,railContacts} from './breadboard.js';
import {wireRuns} from './wires.js';
import {benchPower} from './power.js';
export {benchPower} from './power.js';

export const analyzerHeader=[['CH0','CH1'],['CH2','CH3'],['CH4','CH5'],['CH6','CH7'],['CLK','GND']];
export const analyzerSource='https://learn.lonelybinary.com/manuals/logic-analyzer/the-kit-in-front-of-you';
export const expectedSignals={pulse:{volts:3.3,hertz:1000,duty:.5,periodSeconds:.001},response:{volts:3.3,edgesPerToggle:1000,toggleSeconds:1,hertz:.5},ground:{volts:0}};

export function checkBenchConnections(layout,{rails='unverified'}={}){
  const parent=new Map(),find=id=>{if(!parent.has(id))parent.set(id,id);if(parent.get(id)!==id)parent.set(id,find(parent.get(id)));return parent.get(id);},join=(a,b)=>parent.set(find(a),find(b));
  const isContact=id=>/^[A-J][0-9]+$/.test(id)||railContacts.some(point=>point.id===id),normalize=id=>isContact(id)?'Breadboard.'+breadboardConnection(id,{rails}).group:id;
  for(const pin of picoPins){const id='Pico.'+(pin.name==='GND'?'GND.'+pin.number:pin.name);join(id,normalize(pin.contact));if(pin.name==='GND')join(id,'Pico.GND');}
  join('TPH2.1','Cora.JA1');join('TPH2.2','Cora.JA2');join('TPH2.3','Cora.JA3');join('TPH2.4','Cora.JA4');join('TPH2.5','Cora.GND');
  join('CH1.tip','CH1.coax');join('CH2.tip','CH2.coax');join('CH1.ground','Scope.GND');join('CH2.ground','Scope.GND');
  const used=new Set(picoPins.map(pin=>pin.contact)),errors=[],leads=[];
  const signalNets=['pulse','response','ground',...(layout.profile==='ddcs'?['uartTx','uartRx']:[])];
  for(const net of signalNets)for(const run of wireRuns(layout.nets[net])){
    for(const end of [run.from,run.to]){if(!end){errors.push('Unaddressed lead on '+net);continue;}if(isContact(end)){if(used.has(end))errors.push('Contact reused: '+end);used.add(end);if(!breadboardConnection(end,{rails}).verified)errors.push('Rail continuity unverified: '+end);}if(/^TPH2\.(6|12)$/.test(end))errors.push('Independent power outputs must not be joined');}
    if(run.from&&run.to){join(normalize(run.from),normalize(run.to));leads.push({id:net+'-'+run.index,net,from:run.from,to:run.to,kind:run.kind});}
  }
  const requirements={pulse:['Pico.GP2','Cora.JA1','Analyzer.CH0','Scope.CH1'],response:['Cora.JA2','Analyzer.CH1',...(layout.probes.ch2?['Scope.CH2']:[])],ground:['Pico.GND.3','Cora.GND','Analyzer.GND','Scope.GND']};
  if(layout.profile==='ddcs'){requirements.uartTx=['Pico.GP0','Cora.JA3'];requirements.uartRx=['Pico.GP1','Cora.JA4'];}
  for(const [net,ends] of Object.entries(requirements))for(const end of ends)if(find(end)!==find(ends[0]))errors.push(net+' disconnected: '+end);
  const roots=Object.entries(requirements).map(([net,ends])=>[net,find(ends[0])]);for(let a=0;a<roots.length;a++)for(let b=a+1;b<roots.length;b++)if(roots[a][1]===roots[b][1])errors.push('Short: '+roots[a][0]+' / '+roots[b][0]);
  for(const supply of ['Pico.3V3_OUT','Pico.VBUS','Pico.VSYS','TPH2.6','TPH2.12'])for(const [net,root] of roots)if(find(supply)===root)errors.push('Supply connected to '+net+': '+supply);
  return {ok:errors.length===0,errors,leads,requirements,power:benchPower,expected:layout.profile==='ddcs'?{step:{source:'FPGA JA1',receiver:'Pico GP2 input',finite:true},direction:{source:'FPGA JA2'},uart:{baud:115200}}:expectedSignals,scope:'Declared ideal connection topology only; not a physical continuity test, electrical device model or board execution.'};
}
