import {sha256} from '/dist/digest.js';
import {mountShell,mountTabs,element as node,actionLink,commandButton,propertyList,viewSwitcher} from '../ui/components.js';
import {benchPower} from './power.js';
import {benchSources as source} from './references.js';
import {lazyPanel} from './lazy-panel.js';
import {signalMap} from './connections.js';
import {syncWorkspaceTabs} from '../device-context.js';
import {parseScopeCapture,comparePulseCapture} from './scope-capture.js';
import {workspaceLifecycle} from '../workspace-lifecycle.js';

async function startHardware(){
mountShell({hardwareOnly:document.body.dataset.hardwareOnly==='true'});syncWorkspaceTabs();
const hardwareOnly=document.body.dataset.hardwareOnly==='true';
if(!document.querySelector('.device-context'))document.querySelector('.shell-context-slot').append(node('h1','Hardware bench','hardware-shell-title'));
const panels=[['bench','Bench'],['connections','Wiring'],['experiment','Replay'],['captures','Captures'],['equipment','Equipment'],['procedure','Setup'],['cora','Board pins']].map(([id,label])=>({id,label,panel:document.getElementById(id)}));
if(!hardwareOnly){const hantek=node('section',undefined,'hardware-3d-page hantek-hardware-workspace');hantek.id='hantek';hantek.hidden=true;document.querySelector('main').append(hantek);panels.push({id:'hantek',label:'Device map',panel:hantek});}
const normalize=id=>({live:'captures',ddcs:'bench',acquisition:'captures',setup:'procedure',fields:'bench'}[id]||id);
const requested=normalize(location.hash.slice(1));
const initial=panels.some(panel=>panel.id===requested)?requested:'bench';
const panel=id=>panels.find(panel=>panel.id===id).panel;
const context=node('div',undefined,'hardware-example-context');context.append(node('span','Example preset · pulse fixture'),node('span','2D/3D reference models · not connected hardware'));
document.getElementById('hardware-tabs').prepend(context);
const captureImport=document.querySelector('.hardware-capture-import');
const lazy={
  cora:lazyPanel(panel('cora'),'Cora pins',async()=>{const {mountCoraBoard}=await import('./cora-board.js');return mountCoraBoard();}),
  bench:lazyPanel(panel('bench'),'Bench',async()=>{const {mountHardwareBench}=await import('/dist/hardware-scene.js');const ddcs=new URLSearchParams(location.search).get('profile')==='ddcs',view=mountHardwareBench({guide:false,camera:'overview',profile:ddcs?'ddcs':null,stateKey:'rea.hardware.bench.v2'+(ddcs?'.ddcs':'')});if(new URLSearchParams(location.search).get('inspect')==='cora'){view.setMode('2d');view.inspectPart('cora');}return view;}),
  experiment:lazyPanel(panel('experiment'),'Experiment',async()=>{const {mountPulseExperiment}=await import('./pulse-experiment-ui.js');return mountPulseExperiment({traceFile:'pulse-experiments-xsim.json'});}),
  procedure:lazyPanel(panel('procedure'),'Procedure',async()=>{const {setupProcedure}=await import('./procedure.js');return setupProcedure({onOpen:id=>tabs.select(normalize(id))});}),
  equipment:lazyPanel(panel('equipment'),'Equipment',async()=>{const {mountEquipment}=await import('./equipment.js');return mountEquipment();}),
  captures:lazyPanel(panel('captures'),'Captures',async()=>({root:captureImport,dispose(){}}))
};
if(!hardwareOnly)lazy.hantek=lazyPanel(panel('hantek'),'Hantek hardware',async()=>{return (await import('./hantek-hardware.js')).mountHantekHardware({project:{hardware_url:'/hardware/hantek-hardware.json'}});});
window.addEventListener('pagehide',event=>{if(!event.persisted)Object.values(lazy).forEach(view=>view.dispose());});
const tabs=mountTabs(document.getElementById('hardware-tabs'),panels,{id:'hardware',label:'Hardware bench views',value:initial,onChange:id=>{history.replaceState(null,'','#'+id);for(const [key,view] of Object.entries(lazy))view.setActive(key===id);if(lazy[id])void lazy[id].load().catch(()=>{});}});
const changePanel=()=>{const id=normalize(location.hash.slice(1));if(panel('bench').isConnected&&panels.some(entry=>entry.id===id))tabs.select(id);};
window.addEventListener('hashchange',changePanel);
window.addEventListener('pagehide',event=>{if(!event.persisted)window.removeEventListener('hashchange',changePanel);});
workspaceLifecycle('/hardware/',{hide(){for(const view of Object.values(lazy))view.setActive(false);},show(){const id=normalize(location.hash.slice(1));tabs.select(panels.some(entry=>entry.id===id)?id:'bench');}});

const connections=panels.find(panel=>panel.id==='connections').panel,connectionHeading=connections.firstElementChild,signals=node('div',undefined,'hardware-connection-body');
while(connectionHeading.nextSibling)signals.append(connectionHeading.nextSibling);
const map=signalMap();signals.querySelector('.hardware-map').replaceWith(map.root);window.addEventListener('pagehide',event=>{if(!event.persisted)map.dispose();});
const power=node('div',undefined,'hardware-connection-body'),usb=node('div',undefined,'hardware-connection-body'),probes=node('div',undefined,'hardware-connection-body'),direct=node('div',undefined,'hardware-connection-body');
function connectionTable(parent,caption,columns,rows){const table=node('table',undefined,'hardware-net-table');table.append(node('caption',caption));const head=node('thead'),headRow=node('tr');for(const text of columns){const cell=node('th',text);cell.scope='col';headRow.append(cell);}head.append(headRow);table.append(head);const body=node('tbody');for(const row of rows){const line=node('tr');row.forEach((text,i)=>{const cell=node(i?'td':'th',text);if(!i)cell.scope='row';line.append(cell);});body.append(line);}table.append(body);parent.append(table);}
connectionTable(power,'Separate supplies · shared signal ground',['Device','Power path','Role'],benchPower.map(item=>[item.device,item.from+' → '+item.cable,item.supply]));
power.append(node('p','Pico GND, Cora GND, analyzer GND and the CH1 probe ground clip share the ground strip. No 3.3 V regulator outputs are joined. The analyzer does not power the breadboard.','hardware-connection-note'));
connectionTable(usb,'Example USB connections · one host per device',['Host','Cable → device','Purpose'],[
 ['Acquisition host','USB-C → Micro-USB-B → Pico 2','Board power · MicroPython serial console'],
 ['Acquisition host','USB-C → Mini-USB data cable → analyzer','Board power · PulseView digital capture'],
 ['Programming / acquisition host','USB-A or USB-C → Micro-USB-B → Cora J12','USB-selected power · Vivado JTAG programming'],
 ['Programming / acquisition host','USB-A or USB-C → USB-B → scope rear panel','Optional Hantek PC capture · separate mains power']
]);
usb.append(node('p','Use data cables, not charge-only cables. Cora USB HOST and scope front USB-A are not PC-link ports. Do not cable the two computers together by USB.','hardware-connection-note'),node('p','Use hosts supported by your programming and acquisition tools. Import recorded waveforms in Captures; live adapters are configured separately. Source clocks remain independent.','hardware-connection-note'));
const refs=node('p',undefined,'hardware-connection-note');for(const [text,url] of [['Cora USB/JTAG reference','https://media.distrelec.com/Web/Downloads/_t/ds/410-370-1_eng_tds.pdf'],['Scope manual · §4','https://www.hantek.com/uploadpic/hantek/files/20250326/DSO2000%20Series%20Digital%20Storage%20Oscilloscope%20User%20Manual20250326154940.pdf'],['Analyzer manual',source.logic]]){const link=node('a',text);link.href=url;link.target='_blank';link.rel='noopener noreferrer';refs.append(link,document.createTextNode(' · '));}usb.append(refs);
connectionTable(probes,'Physical connections · low-voltage bench',['Connector','How it attaches','What it measures'],[
 ['Scope probe','Normal probe BNC locks onto CH1; hook grips the E55 jumper pin','CH1 pulse · CH2 unused'],
 ['Probe ground','The normal probe ground clip attach only to common GND','Earth-referenced return, not another signal input'],
 ['Analyzer','Female jumper ends over small-adapter male posts 0 / 1 / GND; male ends in D55 / D60 / C45','CH0 pulse · CH1 response · GND reference'],
 ['Pmod TPH2','12-pin pass-through seats on Cora JA; align pin 1 markings','Pin 1 pulse · pin 2 response · pin 5 GND'],
 ['Breadboard / Pico','Pico headers straddle the center gap; jumpers use shared five-hole strips','Signal tie points; opposite sides of the center gap stay separate']
]);
probes.append(node('p','Match ×10 probe and channel settings. The normal probe ground clip go to common GND. TPH2 is a passive test header, not a level shifter. The virtual scope computes a probe/input equivalent; it is not a live hardware capture.','hardware-connection-note'));
connectionTable(direct,'What plugs into the breadboard',['Part','Direct attachment','In this setup'],[
 ['Pico 2H','Fitted headers straddle the center gap','Mounted · pin 1 at C23; use adjacent free holes'],
 ['Analyzer breadboard adapter','Supplied right-angle adapter mates with the analyzer’s 2×5 header','Fitted · use labelled male posts 0, 1 and GND'],
 ['USB-C power module','Pins enter the rail pairs at one end; match + and −','Not connected · Pico and Cora have their own USB supplies'],
 ['Kit LEDs, resistors and capacitors','Through-hole leads enter separate terminal strips','Optional · LED needs a series resistor; observe polarity']
]);
direct.append(node('p','This example uses three male–female analyzer jumpers. The adapter exposes signals; it does not power the breadboard.','hardware-connection-note'),node('p','TPH2 plugs into Cora JA, not the breadboard. The 12 mm button and AS5600 module need matching lead/header spacing before direct insertion.','hardware-connection-note'));
const directRefs=node('p',undefined,'hardware-connection-note');for(const [text,url] of [['Analyzer adapter',source.logic+'/the-kit-in-front-of-you'],['Power module','https://learn.lonelybinary.com/manuals/bbpwd'],['Pico pinout','https://datasheets.raspberrypi.com/pico/pico-2-datasheet.pdf']]){const link=node('a',text);link.href=url;link.target='_blank';link.rel='noopener noreferrer';if(directRefs.childNodes.length)directRefs.append(document.createTextNode(' · '));directRefs.append(link);}direct.append(directRefs);
const connectionViews={signals,power,usb,probes,direct};const connectionMode=viewSwitcher({label:'Connection details',items:[['signals','Signals'],['power','Power'],['usb','USB'],['probes','Probes'],['direct','Connectors']],value:'signals',onChange:id=>{for(const [key,panel] of Object.entries(connectionViews))panel.hidden=key!==id;}});power.hidden=true;usb.hidden=true;probes.hidden=true;direct.hidden=true;connections.append(connectionMode,signals,power,usb,probes,direct);

const fileInput=document.getElementById('capture-file'),status=document.getElementById('capture-status'),record=document.getElementById('capture-record'),actions=document.getElementById('capture-actions');
let receipt=null,capture=null;
const save=commandButton('Export receipt',{kind:'primary'});save.disabled=true;actions.append(save);
const replay=commandButton('View on scope',{kind:'secondary'});replay.disabled=true;actions.append(replay);replay.addEventListener('click',async()=>{if(!capture)return;const selected=capture;replay.disabled=true;tabs.select('bench');try{const bench=await lazy.bench.load();await bench.ready;if(capture===selected)bench.loadScopeCapture(selected);}catch{status.textContent='Scope view could not be opened. Retry.';}finally{replay.disabled=!capture;}});
const compare=commandButton('Compare with example pulse',{kind:'ghost'});compare.disabled=true;actions.append(compare);compare.addEventListener('click',()=>{if(!capture||!receipt)return;receipt.comparison=comparePulseCapture(capture);record.append(propertyList([['Example 1 kHz comparison',receipt.comparison.status]]));compare.disabled=true;});
const clear=commandButton('Remove',{kind:'ghost'});clear.hidden=true;actions.append(clear);clear.addEventListener('click',()=>{receipt=capture=null;fileInput.value='';record.replaceChildren();save.disabled=replay.disabled=compare.disabled=true;clear.hidden=true;status.textContent='No capture attached.';});
fileInput.addEventListener('change',async()=>{
  receipt=capture=null;save.disabled=replay.disabled=compare.disabled=true;clear.hidden=true;record.replaceChildren();const file=fileInput.files[0];if(!file){status.textContent='No capture attached.';return;}fileInput.disabled=true;
  try{
    if(file.size===0||file.size>64*1024*1024)throw new Error('Choose a nonempty capture up to 64 MiB.');status.textContent='Reading capture…';const bytes=await file.arrayBuffer(),fileSha256=await sha256(bytes);
    receipt={schema:1,file:{name:file.name,bytes:file.size,sha256:fileSha256},state:'file identified locally; provenance unverified',missing:['instrument identity and firmware','physical wiring check','programmed image hashes','probe and acquisition settings']};
    record.append(propertyList([['File',file.name],['SHA-256',fileSha256]]));
    if(/\.(csv|tsv|json)$/i.test(file.name)){
      try{capture=parseScopeCapture(new TextDecoder().decode(bytes));receipt.waveform={format:capture.format,origin:capture.declaredOrigin,samples:capture.samples,spanSeconds:capture.span,minVolts:capture.min,maxVolts:capture.max,frequencyHz:capture.frequency,duty:capture.duty};record.append(propertyList([['Data',capture.declaredOrigin],['Samples',String(capture.samples)],['Frequency',capture.frequency===null?'Insufficient rising edges':capture.frequency.toFixed(3)+' Hz'],['Peak-to-peak',capture.vpp.toFixed(3)+' V']]));replay.disabled=compare.disabled=false;status.textContent='Waveform parsed · provenance unverified';}catch(error){receipt.waveformError=error.message;status.textContent='File retained · '+error.message;}
    }else status.textContent='File identified · this format has no waveform reader';
    save.disabled=false;clear.hidden=false;
  }catch(error){status.textContent=error.message;}finally{fileInput.disabled=false;}
});
save.addEventListener('click',()=>{if(!receipt)return;const url=URL.createObjectURL(new Blob([JSON.stringify(receipt,null,2)+'\n'],{type:'application/json'})),link=node('a');link.href=url;link.download='hardware-capture-receipt.json';link.click();setTimeout(()=>URL.revokeObjectURL(url),1000);});
}
void startHardware().catch(error=>{const main=document.querySelector('main');main.replaceChildren(node('h1','Hardware unavailable'),node('p',error.message));});
