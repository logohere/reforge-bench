import {sha256} from '/dist/digest.js';
import {element as node,commandButton,badge} from '../ui/components.js';
import {benchSources} from './references.js';

export const procedureSteps=[
  {label:'Equipment',title:'Choose your equipment',note:'Pulse fixture example · substitute equipment after checking interfaces and ratings',refs:['cora','pico','tph'],open:['equipment','Edit equipment list'],cards:[
    {title:'Stimulus',lines:[['Role','Generate a known input'],['Example','Pico 2 · 3.3 V GPIO']],body:'Use a source compatible with your target. Keep its output disabled while wiring.'},
    {title:'Target',lines:[['Role','Process the input'],['Example','Cora Z7-07S + TPH2']],body:'The example counts 1,000 input edges per return transition. Other targets need their own pin map and program.'},
    {title:'Measurement',lines:[['Analog','Oscilloscope'],['Digital','Logic analyzer']],body:'Use instruments suited to the voltage and timing. No particular scope, host computer or acquisition adapter is required.'}
  ]},
  {label:'Build',title:'Check the example program',note:'Retained RTL simulation is not a board-programming receipt',refs:['constraints','pico'],open:['experiment','Open replay'],files:[['RTL','pulse_loop.v'],['Testbench','pulse_loop_tb.v'],['Constraints','pulse_loop.xdc'],['Stimulus script','pulse_source.py'],['RTL receipt','rtl-check.json']],cards:[
    {title:'Simulate',lines:[['Input','1,000 rising edges'],['Return','Toggle once']],body:'Replay the nominal, missing-input, stuck-high and reset cases. Keep simulation separate from measurements.'},
    {title:'Build for your target',lines:[['Example','Cora 07S Rev. B'],['Inputs','RTL + matching constraints']],body:'Use your supported FPGA toolchain. Review timing and pin assignments before programming; retain the resulting artifact hashes.'}
  ]},
  {label:'Wire',title:'Wire with outputs disabled',note:'These pin assignments belong to the example preset only · 3.3 V logic',refs:['tph','constraints'],open:['connections','Open wiring'],cards:[
    {title:'Pulse',lines:[['Source','Pico GP2 · pin 4'],['Target','TPH2 pin 1 / Cora JA1']],body:'Observe the same pulse net with scope CH1 and analyzer CH0.'},
    {title:'Return',lines:[['Target','TPH2 pin 2 / Cora JA2'],['Observe','Analyzer CH1']],body:'Connect the return separately from the pulse input.'},
    {title:'Ground',lines:[['Reference','Pico GND ↔ TPH2 GND'],['Observe','Instrument grounds']],body:'Use one common reference. Do not join regulator outputs. Verify actual pin numbering, probe ratings and ground topology.'}
  ]},
  {label:'Program',title:'Program and verify',note:'Use your own programming tools; Reforge does not flash a device',refs:['cora','pico'],cards:[
    {title:'Program the target',lines:[['Before','Stimulus disabled'],['Verify','Target identity + artifact hash']],body:'Follow the board’s programming procedure. Check configuration status and keep supply outputs isolated.'},
    {title:'Enable the stimulus',lines:[['Example','1 kHz · 50% duty'],['Return','One transition per 1,000 edges']],body:'Enable the source only after wiring and supply checks. Independently measure the requested frequency and voltage.'}
  ]},
  {label:'Capture',title:'Capture the signals',note:'Example scope settings, not acquired measurements',refs:['scopeManual','logic'],open:['captures','Open captures'],cards:[
    {title:'Analog',lines:[['Example','CH1 · DC · 1 V/div'],['Timebase','200 µs/div'],['Trigger','Rising edge · near 1.65 V']],body:'Match probe attenuation to the channel setting. Set range, grounding and trigger for your actual equipment.'},
    {title:'Digital',lines:[['Channels','Pulse + return'],['Example rate','1 MS/s']],body:'Record with your acquisition tool or configured adapter. Import supported waveforms in Captures; retain originals and settings.'}
  ]},
  {label:'Compare',title:'Inspect the result',note:'Capture timestamps and graph relationships do not prove synchronized clocks',refs:['logic'],open:['captures','Open captures'],cards:[
    {title:'Retain evidence',lines:[['Identity','Original file + hash'],['Context','Settings + pin map + program version']],body:'Keep measured, simulated and reference data distinct. Record missing samples and failed checks.'},
    {title:'Fault and recovery',lines:[['Example','Stop then restore the pulse'],['Expected','Watchdog fault then recovery']],body:'Use the replay to inspect expected behavior. Physical behavior requires a corresponding hardware capture.'}
  ]}
];

export async function loadRtlReceipt({request=fetch,signal}={}){
  const response=await request('/hardware/rtl-check.json',{cache:'no-store',signal});
  if(!response.ok)throw new Error('RTL receipt unavailable');
  const receipt=await response.json(),paths=['pulse_loop.v','pulse_loop_tb.v','pulse_loop.xdc','pulse_source.py','sensor_source.py'].map(file=>'web/hardware/'+file);
  if(receipt.ok!==true||typeof receipt.result!=='string'||!receipt.result.startsWith('PASS:')||!receipt.sourceSha256||Object.keys(receipt.sourceSha256).length!==paths.length||paths.some(path=>!/^[a-f0-9]{64}$/.test(receipt.sourceSha256[path]||'')))throw new Error('RTL receipt is invalid');
  await Promise.all(paths.map(async path=>{const source=await request('/'+path.slice(4),{cache:'no-store',signal});if(!source.ok)throw new Error('RTL source unavailable');if(await sha256(await source.arrayBuffer())!==receipt.sourceSha256[path])throw new Error('RTL receipt is stale');}));
  return receipt;
}

export function setupProcedure({initial=0,loadRtl=loadRtlReceipt,onOpen=null}={}){
  if(!Number.isInteger(initial)||initial<0||initial>=procedureSteps.length)throw new RangeError('Unknown procedure step');
  const root=node('div',undefined,'hardware-procedure'),nav=node('nav',undefined,'hardware-procedure-list'),list=node('ol'),detail=node('article',undefined,'hardware-step-detail'),header=node('header'),heading=node('h2'),check=node('div',undefined,'hardware-procedure-check'),content=node('div',undefined,'hardware-procedure-content'),footer=node('footer'),back=commandButton('Back',{kind:'secondary'}),position=node('span'),next=commandButton('Next step',{kind:'primary'}),buttons=[];
  const abort=new AbortController();let active=initial,rtlState='idle',rtlError='',pending=null,disposed=false;
  setupProcedure.sequence=(setupProcedure.sequence||0)+1;heading.id='hardware-procedure-title-'+setupProcedure.sequence;heading.tabIndex=-1;detail.setAttribute('aria-labelledby',heading.id);nav.setAttribute('aria-label','Bring-up sequence');nav.append(node('h2','Bring-up sequence'),list,node('p','Phase 1 · Pico / Cora pulse loop','hardware-procedure-context'));header.append(heading,check);footer.append(back,position,next);detail.append(header,content,footer);root.append(nav,detail);
  function paintCheck(){
    if(disposed)return;check.replaceChildren();
    if(active!==1){check.append(badge(active===0?'Equipment list':'Plan only',active===0?'info':'warning'));return;}
    const status=badge(rtlState==='checked'?'RTL checked · no bitstream':rtlState==='error'?rtlError:rtlState==='loading'?'Checking RTL sources…':'RTL not checked',rtlState==='error'?'danger':'info');status.setAttribute('role',rtlState==='error'?'alert':'status');check.append(status);
    if(rtlState==='error'){const retry=commandButton('Retry',{kind:'ghost',size:'sm'});retry.addEventListener('click',()=>void checkRtl());check.append(retry);}
  }
  function checkRtl(){
    if(pending||rtlState==='checked'||disposed)return pending;
    rtlState='loading';paintCheck();pending=Promise.resolve().then(()=>loadRtl({signal:abort.signal})).then(()=>{rtlState='checked';}).catch(error=>{rtlState='error';rtlError=error.message;}).finally(()=>{pending=null;if(active===1)paintCheck();});return pending;
  }
  function link(label,href,download){const item=node('a',label);item.href=href;if(download)item.download=download;else{item.target='_blank';item.rel='noopener noreferrer';}return item;}
  function select(index,focus=false){
    if(disposed)return;if(!Number.isInteger(index)||index<0||index>=procedureSteps.length)throw new RangeError('Unknown procedure step');active=index;root.dataset.step=String(index);const step=procedureSteps[index];heading.textContent=step.title;position.textContent=String(index+1).padStart(2,'0')+' / '+String(procedureSteps.length).padStart(2,'0');back.disabled=index===0;next.textContent=index===5?'Open Captures':'Next step';next.disabled=index===5&&!onOpen;content.replaceChildren();
    const cards=node('div',undefined,'hardware-procedure-cards');
    for(const item of step.cards){const card=node('section',undefined,'hardware-procedure-card');if(item.image){const [file,alt,caption]=item.image,figure=node('figure'),image=node('img'),label=node('figcaption',caption);image.src='/assets/hardware/'+file;image.alt=alt;image.loading='lazy';image.decoding='async';figure.append(image,label);card.append(figure);}else{const diagram=node('dl',undefined,'hardware-procedure-diagram');for(const [label,value] of item.lines){const row=node('div');row.append(node('dt',label),node('dd',value));diagram.append(row);}card.append(diagram);}card.append(node('h3',item.title),node('p',item.body));cards.append(card);}
    content.append(node('p',step.note,'hardware-procedure-note'),cards);

    const resources=node('div',undefined,'hardware-procedure-resources');resources.setAttribute('role','group');resources.setAttribute('aria-label','Step resources');
    if(step.files){const files=node('div');files.append(node('span','Files'));for(const [label,file] of step.files)files.append(link(label,'/hardware/'+file,file));resources.append(files);}
    const refs=node('div');refs.append(node('span','References'));for(const key of step.refs)refs.append(link({cora:'Cora',pico:'Pico',tph:'TPH2',scope:'Scope',scopeManual:'Scope manual',logic:'Analyzer',constraints:'Pin constraints',sensor:'AS5600'}[key],benchSources[key]));resources.append(refs);
    if(step.open&&onOpen){const open=commandButton(step.open[1],{kind:'ghost',size:'sm'});open.addEventListener('click',()=>onOpen(step.open[0]));resources.append(open);}content.append(resources);
    buttons.forEach((button,i)=>{button.dataset.active=String(i===index);void button.updateComplete.then(()=>{const control=button.shadowRoot.querySelector('button');if(i===active)control?.setAttribute('aria-current','step');else control?.removeAttribute('aria-current');});});paintCheck();if(index===1)void checkRtl();if(focus)heading.focus();
  }
  procedureSteps.forEach((step,index)=>{const button=commandButton(String(index+1).padStart(2,'0')+'  '+step.label,{kind:'ghost'}),item=node('li');button.addEventListener('click',()=>select(index));buttons.push(button);item.append(button);list.append(item);});
  back.addEventListener('click',()=>select(active-1,true));next.addEventListener('click',()=>{if(active===5)onOpen?.('captures');else select(active+1,true);});select(initial);
  return {root,select,dispose(){disposed=true;abort.abort();}};
}
