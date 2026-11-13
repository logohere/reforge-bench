import {element as node,commandButton,propertyList,actionLink,setPressed} from '../ui/components.js';
import {pulseLoopContacts} from './breadboard.js';
import {coraModel} from './cora-model.js';
import {tphModel} from './tph-model.js';

export const coraJaPins=[
  {pin:1,label:'P1',role:'Pulse input',package:'Y18'},
  {pin:2,label:'P2',role:'Return output',package:'Y19'},
  {pin:3,label:'P3',role:'Unused',package:'Y16'},
  {pin:4,label:'P4',role:'Unused',package:'Y17'},
  {pin:5,label:'GND',role:'Common ground'},
  {pin:6,label:'VCC',role:'3.3 V · leave unused'},
  {pin:7,label:'P7',role:'Unused',package:'U18'},
  {pin:8,label:'P8',role:'Unused',package:'U19'},
  {pin:9,label:'P9',role:'Unused',package:'W18'},
  {pin:10,label:'P10',role:'Unused',package:'W19'},
  {pin:11,label:'GND',role:'Common ground'},
  {pin:12,label:'VCC',role:'3.3 V · leave unused'}
];
export function mountCoraBoard(){
  const root=node('section',undefined,'cora-board'),heading=node('header'),body=node('div',undefined,'cora-board-body'),figure=node('figure'),caption=node('figcaption'),right=node('section',undefined,'cora-board-pins'),detail=node('div',undefined,'cora-board-detail'),grid=node('div',undefined,'cora-pin-grid'),buttons=[];
  heading.append(node('h2','Cora Z7-07S · board and Pmod pins'),node('p','Digilent top-view orientation · Rev B reference fixture'));
  const ns='http://www.w3.org/2000/svg',make=(tag,attributes,text)=>{const element=document.createElementNS(ns,tag);for(const [key,value] of Object.entries(attributes))element.setAttribute(key,String(value));if(text!==undefined)element.textContent=text;return element;},drawing=make('svg',{viewBox:'-56 -36 112 72',role:'img','aria-label':'Modeled top view of Cora Z7 with labeled ports and controls',class:'cora-board-drawing'});drawing.append(coraModel(make));
  caption.textContent='Reference model · match connector names and the physical board silkscreen.';figure.append(drawing,caption);
  right.append(node('h3','TPH2 test posts · read the printed labels'),node('p','Two rows of six posts: P1–P4 / GND / VCC and P7–P10 / GND / VCC. P10 means pin ten. This number map is a schematic; match the silkscreen before connecting.'));
  const pmodDrawing=make('svg',{viewBox:'-25 -12 48 24',role:'img','aria-label':'Enlarged TPH2 male plug, female pass-through, and labelled upright test posts',class:'tph-board-drawing'});pmodDrawing.append(tphModel(make));right.append(pmodDrawing,node('p','Male plug toward Cora JA. Clip or slide female jumper ends onto the upright posts. The outer female connector is the pass-through, not the test posts.'));
  grid.setAttribute('role','group');grid.setAttribute('aria-label','TPH2 numbered test posts, with P1 end at the top');
  function select(pin){const item=coraJaPins.find(item=>item.pin===pin);for(const [number,button] of buttons)setPressed(button,number===pin);const ground=pulseLoopContacts.ground,pulse=pulseLoopContacts.pulse,response=pulseLoopContacts.response;const path=pin===1?`Pico GP2 / pin 4 → ${pulse.source} → ${pulse.feed}; ${pulse.output} → P1 → JA1 / Y18`:pin===2?`JA2 / Y19 → P2 → ${response.output}; ${response.analyzer} → analyzer CH1`:pin===5||pin===11?`Pico GND / pin 3 → ${ground.source} → ${ground.feed}; ${ground.output} → TPH2 GND (pin 5). Instrument grounds share A45–E45.`:pin===6||pin===12?'Leave this 3.3 V supply post unconnected. Pico and Cora use their own USB power; their regulator outputs are not joined.':'Leave this signal unused for the pulse fixture.';detail.replaceChildren(node('h3',item.label+' · connector pin '+pin),propertyList([['Role',item.role],['FPGA package',item.package||'Board power / ground']]),node('p',path));}
  for(let row=0;row<6;row++)for(const number of [row+1,row+7]){const pin=coraJaPins[number-1],button=commandButton(pin.label+' · '+pin.role,{kind:'ghost',size:'sm'});button.dataset.pin=String(number);button.addEventListener('click',()=>select(number));buttons.push([number,button]);grid.append(button);}
  const controls=node('details'),summary=node('summary','Board controls and LEDs');controls.append(summary,propertyList([['J12 · callout 3','Micro-USB JTAG / UART; Programming host and USB power'],['JP3 · callout 1','USB / external power selector; inspect the actual jumper'],['JA · callout 5','TPH2 and this pulse fixture; JB remains unused'],['BTN0 · callout 10','Fixture reset; not SRST or the power-on reset button'],['LD0 · callout 9','PWM-limited red fault / green heartbeat indication'],['LD7 · callout 17','Red power-good indicator'],['LD6 · callout 13','Configuration DONE indicator; does not measure pulse operation']]));
  right.append(grid,detail,controls);body.append(figure,right);const footer=node('footer');footer.append(node('p','If TPH2 is already fitted, leave it fitted. Otherwise unplug Cora USB and fit the already-wired adapter. Keep the harness connected, stop Pico stimulus, then power Cora and use wired-stimulus-disabled programming. Power loss clears the volatile image.'),actionLink('Step-by-step wiring','/hardware/#bench'),actionLink('Setup guide','/hardware/#procedure'));
  const sources=node('details'),sourceTitle=node('summary','Manufacturer references');sources.append(sourceTitle,actionLink('Cora reference manual','https://media.digikey.com/pdf/Data%20Sheets/Digilent%20PDFs/Cora_Z7_RM_Web.pdf'),actionLink('TPH2 reference manual','https://digilent.com/reference/_media/reference/pmod/pmodtph2/pmodtph2_rm.pdf'));
  root.append(heading,body,footer,sources);select(1);return {root,dispose(){}};
}
