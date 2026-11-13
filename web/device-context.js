import {analysisAction} from './analysis-action.js';
import {analysisProgress} from './library/progress.js';
import {systemCase,deviceQuery} from './system-routes.js';
import {element as node,badge,mountShell,icon,observeShell,evidenceStages,actionLink,commandButton} from './ui/components.js';
import {systemInfo,systemGroups,manufacturerGroups} from './ui/system-catalog.js';
import {loadProjectLibrary,projectForCase} from './device-project.js';
export const DEVICE_KEY='rea.active-device.v1';
export const DDCS_SHA='c70ff4eceadb98c811ccfe81efd0e62bd3d6c13e5ac18dfb8b590f75790f9b05';
export let activeDevice=null;

export function requestedDevice(search,storage){
  const requested=systemCase(search);
  if(requested!==null)return requested;
  try{return storage().getItem(DEVICE_KEY)||null;}catch{return null;}
}

export function deviceEvidence(item,recovery){
  const record=recovery?.[item?.profile_id];
  return record?.sha256===item?.case_id?record:null;
}

export function deviceRoutes(item,recovery){
  const id=item?.case_id;
  if(!/^[a-f0-9]{64}$/.test(id||''))return null;
  const query=deviceQuery(id),record=deviceEvidence(item,recovery);
  let process='/rea/'+query+'#target';
  if(record&&item.profile_id==='ddcs-expert-m350-motiondev')process='/rea/'+query+'&driver=expert-m350#kernel';
  if(record&&item.profile_id==='ddcs-v41-motiondev')process='/rea/'+query+'&driver=ddcs-v41#kernel';
  return {home:'/home/'+query,workflow:'/workflow/'+query,library:'/library/'+query,process,timeline:'/timeline/'+query,lab:'/lab/'+query,help:'/help/'+query,hardware:'/hardware/'+query,
    haas:['7b33f8e7f5f290c34e9b21eb4027eab6ab4a5fbea7b725b9c7974fcd82c15171','148fde6e22ee1e7ef6e16fb45c5bd70776feb53c20979a4b6e9289162461b9d7','e9b9039e1241a54233033ffb5bfe135c7fc3d177d3793e8df3ff8a25f123a898'].includes(id)&&!!record,
    ddcs:record?.lab_contract?.firmware_sha256===id&&item.profile_id==='ddcs-expert-2020'&&id===DDCS_SHA,
    walkthrough:!!record&&((item.profile_id==='ddcs-expert-2020'&&id===DDCS_SHA)||['ddcs-expert-m350-motiondev','ddcs-v41-motiondev'].includes(item.profile_id))};
}

const storage=()=>sessionStorage;
let contextBar;


export function syncWorkspaceTabs(){
  const surface=document.body.dataset.workspace;
  const current=surface==='process'?(location.hash==='#timeline'?'timeline':'rea'):surface;
  for(const link of document.querySelectorAll('.workspace-tabs [data-surface]')){
    const active=link.dataset.surface===current;link.active=active;
    if(active)link.setAttribute('aria-current','page');else link.removeAttribute('aria-current');
    if(link.updateComplete)void link.updateComplete.then(()=>{const anchor=link.shadowRoot?.querySelector('a');if(active)anchor?.setAttribute('aria-current','page');else anchor?.removeAttribute('aria-current');});
  }
  for(const link of document.querySelectorAll('.shell-help')){
    if(new URL(link.href,location.href).pathname===location.pathname)link.setAttribute('aria-current','page');else link.removeAttribute('aria-current');
  }
}

export function labRunMismatch(run,contract,programSha=contract?.program.sha256){
  if(!run.run_id)return null;
  if(!contract)return 'No pinned firmware/hardware contract for this Lab.';
  const expectedImage=run.build==='original'?contract.firmware_sha256:contract.hybrid_sha256;
  for(const [field,expected] of [['firmware_sha256',contract.firmware_sha256],['executable_sha256',expectedImage],['setting_sha256',contract.settings.sha256],['program_sha256',programSha]]){
    if((run[field]||run.status==='complete')&&run[field]!==expected)return `Lab identity mismatch: ${field}.`;
  }
  return null;
}

export function selectDevice(item,cases,recovery){
  mountShell();
  const routes=deviceRoutes(item,recovery),info=systemInfo(item);
  if(!routes)throw new Error('Device identity unavailable.');
  document.title=`REA — ${info.vendor} ${info.name} · ${document.body.dataset.workspace}`;
  try{storage().setItem(DEVICE_KEY,item.case_id);}catch{}
  window.dispatchEvent(new CustomEvent('rea-device-selected',{detail:item.case_id}));
  document.querySelector('.app-header .brand').href='/home/';
  for(const link of document.querySelectorAll('.workspace-tabs [data-surface]')){
    const surface=link.dataset.surface;
    if(surface==='library')link.href=routes.library;
    else if(surface==='rea')link.href=routes.process;
    else if(surface==='timeline')link.href=routes.timeline;
    else if(surface==='lab')link.href=routes.lab;
    else if(surface==='hardware')link.href=routes.hardware;
  }
  const help=document.querySelector('.shell-help:not(.shell-guides)');if(help)help.href=routes.help;
  contextBar?.remove();document.getElementById('system-picker-dialog')?.remove();
  contextBar=node('div',undefined,'device-context');
  const picker=node('button',undefined,'system-switch');picker.type='button';picker.setAttribute('aria-haspopup','dialog');
  const label=node('span',undefined,'system-switch-label');label.append(node('strong',info.shortName||info.name),node('span',info.purpose));picker.title=info.name;
  picker.append(label,icon('chevron--down'));picker.setAttribute('aria-label','Choose system: '+info.name);
  const version=node('span',info.version,'system-version');
  const selectedRoutes=deviceRoutes(item,recovery),supported=selectedRoutes.ddcs;
  const project=projectForCase(item);
  const support=badge(project?.capability_label||(supported?'Research emulation':selectedRoutes.haas?'Bounded Haas research':'Investigation only'),project?'success':supported||selectedRoutes.haas?'info':'neutral');
  const load=node('a','Open firmware','device-load');load.href=routes.library+'&import=1';
  contextBar.append(picker,version,support,load);document.querySelector('.shell-context-slot').append(contextBar);observeShell();
  const dialog=node('dialog',undefined,'system-dialog');dialog.id='system-picker-dialog';dialog.setAttribute('aria-labelledby','system-picker-title');
  const heading=node('div',undefined,'dialog-heading'),title=node('h2','Choose a system');title.id='system-picker-title';
  const close=commandButton('Close',{kind:'ghost'});close.className='dialog-close';close.addEventListener('click',()=>dialog.close());heading.append(title,close);
  const search=node('cds-text-input');search.label='Search systems';search.size='md';search.placeholder='Controller or manufacturer';
  const list=node('div',undefined,'system-options'),empty=node('p','No matching systems. Change or clear the search.','system-empty');empty.setAttribute('role','status');empty.hidden=true;
  const groups=systemGroups(cases,item.case_id).sort((a,b)=>Math.max(...b.items.map(item=>analysisProgress(item,recovery).rank))-Math.max(...a.items.map(item=>analysisProgress(item,recovery).rank))||a.vendor.localeCompare(b.vendor)||a.name.localeCompare(b.name));
  const buckets=new Map();for(const brand of manufacturerGroups(groups)){const section=node('section',undefined,'system-brand');section.append(node('h3',brand.vendor));for(const family of brand.families){const group=node('div',undefined,'system-family');group.append(node('p',family.name));section.append(group);const inactive=family.groups.filter(member=>!member.items.some(candidate=>candidate.case_id===item.case_id));let others;if(inactive.length>1){others=node('details',undefined,'system-other-options');others.append(node('summary',`Other ${family.name} systems · ${inactive.length}`));group.append(others);}for(const member of family.groups)buckets.set(member.item.case_id,others&&inactive.includes(member)?others:group);}list.append(section);}
  const options=[];
  for(const group of groups){
    const links=deviceRoutes(group.item,recovery);if(!links)continue;
    const surface=document.body.dataset.workspace;
    const link=node('a',undefined,'system-option');link.href=links[surface]||links.lab;
    if(group.items.some(candidate=>candidate.case_id===item.case_id))link.setAttribute('aria-current','true');
    const identity=node('div',undefined,'system-option-identity');identity.append(node('strong',group.vendor+' · '+group.name),node('span',group.purpose));
    const metadata=node('div',undefined,'system-option-meta');metadata.append(node('span',group.version),badge(group.item.device_project?.capability_label||(links.ddcs?'Emulation available':links.haas?'Bounded Haas research':'Investigation only'),group.item.device_project?'success':links.ddcs||links.haas?'info':'neutral'));
    link.append(identity,metadata);buckets.get(group.item.case_id).append(link);options.push({link,routes:links,search:(group.vendor+' '+group.name+' '+group.purpose+' '+group.version).toLowerCase()});
  }
  for(const details of list.querySelectorAll('.system-other-options'))details.parentElement.append(details);
  search.addEventListener('input',()=>{for(const details of list.querySelectorAll('.system-other-options'))details.open=!!search.value.trim();for(const option of options)option.link.hidden=!option.search.includes(search.value.trim().toLowerCase());empty.hidden=options.some(option=>!option.link.hidden);for(const group of list.querySelectorAll('.system-family'))group.hidden=![...group.querySelectorAll('.system-option')].some(link=>!link.hidden);for(const brand of list.querySelectorAll('.system-brand'))brand.hidden=![...brand.querySelectorAll('.system-family')].some(group=>!group.hidden);});
  search.addEventListener('keydown',event=>{if(event.key==='ArrowDown'){event.preventDefault();options.find(option=>!option.link.hidden&&(!option.link.closest('.system-other-options')||option.link.closest('.system-other-options').open))?.link.focus();}});
  dialog.append(heading,search,list,empty);document.body.append(dialog);
  picker.addEventListener('click',()=>{const surface=document.body.dataset.workspace;for(const option of options)option.link.href=option.routes[surface]||option.routes.lab;dialog.showModal();search.focus();});
  dialog.addEventListener('click',event=>{if(event.target===dialog){const rect=dialog.getBoundingClientRect();if(event.clientX<rect.left||event.clientX>rect.right||event.clientY<rect.top||event.clientY>rect.bottom)dialog.close();}});
  dialog.addEventListener('close',()=>picker.focus());
  activeDevice={item,recovery,record:deviceEvidence(item,recovery),routes,project};syncWorkspaceTabs();return activeDevice;
}

export function showLabContract(context){
  const contract=context.record?.lab_contract,host=document.getElementById('lab-contract');
  if(!host||!contract)return;
  host.replaceChildren(node('summary','System evidence'));
  const list=node('dl',undefined,'device-contract');
  for(const [label,value] of [['Firmware',contract.firmware_sha256],['Device ABI',contract.device_abi],['Runtime',contract.runtime],['Logic',contract.logic],['Geometry',contract.geometry],['Compared',`${contract.comparison.records.toLocaleString()} records · square/M30`]])list.append(node('dt',label),node('dd',value));
  host.append(list,node('p',contract.scope));
  const details=node('details');details.append(node('summary','Pinned construction artifacts'),node('pre',JSON.stringify(contract,null,2),'contract-json'));host.append(details);
}

export async function loadDeviceContext(){
  mountShell();
  const requested=requestedDevice(location.search,storage),id=requested===null?DDCS_SHA:requested;
  if(document.body.dataset.router==='ready'&&activeDevice?.item.case_id===id){const info=systemInfo(activeDevice.item);document.title=`REA — ${info.vendor} ${info.name} · ${document.body.dataset.workspace}`;syncWorkspaceTabs();return activeDevice;}
  const [response,result]=await Promise.all([fetch('/api/library',{cache:'no-store',signal:AbortSignal.timeout(10000)}),fetch('/library/recovery-status.json',{cache:'no-store',signal:AbortSignal.timeout(10000)})]);
  const data=await response.json();
  if(!response.ok||!data.ok)throw new Error(data.error||'Device library unavailable.');
  if(!result.ok)throw new Error('Device evidence unavailable.');
  const recovery=(await result.json()).profiles;
  const cases=await loadProjectLibrary(data.cases);
  const item=cases.find(candidate=>candidate.case_id===id);
  if(!item)throw new Error('Selected firmware is not in this library. Choose a device in Library.');
  return selectDevice(item,cases,recovery);
}

export function deviceOverview(context,surface){
  const {item,record,routes}=context,metadata=item.library;
  const panel=node('section',undefined,'device-overview');
  if(surface==='lab'){
    const info=systemInfo(item);panel.classList.add('runtime-unavailable');
    panel.append(badge('Investigation only'),node('h1','Simulation not available yet'),node('p',info.vendor+' '+info.name+' · '+info.version,'device-eyebrow'),node('p','The firmware files are saved, but this controller software cannot run in the Lab yet. Continue its investigation or inspect the package.','device-description'));
    const actions=node('div',undefined,'device-actions');for(const [label,href] of [['Open REA',routes.process],['View package files',routes.library]])actions.append(actionLink(label,href,{primary:label==='Open REA'}));panel.append(actions);
    const detail=node('details');detail.append(node('summary','What is missing'),node('p',record?.gaps||'Recover the executable format, target hardware and device interface, then build and verify a target-specific adapter.'));panel.append(detail);return panel;
  }
  panel.append(node('p',`${metadata.vendor||'Unknown manufacturer'} · ${item.original_filename}`,'device-eyebrow'),node('h1',metadata.model||'Unidentified firmware'));
  const blocked=record?.milestones?.some(([label,value])=>label==='Analyze'&&value==='Blocked');
  const progress=analysisProgress(item,context.recovery),analyzed=item.last_runs?.analyze?.ok===true;
  const stateLabel=badge(blocked?'Analysis blocked':progress.label,blocked?'warning':progress.tone);panel.append(stateLabel);
  if(record?.description)panel.append(node('p',record.description,'device-description'));
  const stages=evidenceStages(record?.milestones||[['Acquire','Saved'],['Analyze',analyzed?'Static evidence':'Not recorded'],['Compare','Not run'],['Execute','No adapter'],['Hardware','Unverified']]);
  if(surface==='process')panel.append(analysisAction(item.case_id));
  panel.append(stages);
  if(surface==='lab'){
    const contract=node('dl',undefined,'device-contract');
    for(const [label,value] of [['Firmware',item.profile_id?'Exact-hash profile':'Unidentified'],['Device ABI','Not integrated'],['Hardware model','Not matched'],['Run / comparison','Not established']])contract.append(node('dt',label),node('dd',value));
    panel.append(contract);
  }
  const history=node('details');history.append(node('summary','Construction record'));
  const entries=node('ol');
  for(const [label,value] of record?.stages||[['Imported',new Date(item.imported_at_ms).toISOString()],[analyzed?'Analyzed':'Next',analyzed?'Source-bound static REA evidence is retained. Open Timeline for the recorded traces and functions.':'Establish format, CPU, device boundary and a target-specific adapter']]){
    const entry=node('li');entry.append(node('strong',label+': '),document.createTextNode(value));entries.append(entry);
  }
  history.append(entries);if(record?.proof)history.append(node('p',record.proof));if(record?.gaps)history.append(node('p',record.gaps));
  const identity=node('dl',undefined,'device-contract');
  for(const [label,value] of [['SHA-256',item.case_id],['Profile',item.profile_id||'None'],['Imported',new Date(item.imported_at_ms).toISOString()]])identity.append(node('dt',label),node('dd',value));
  history.append(identity);if(item.verified_scope)history.append(node('p',item.verified_scope));panel.append(history);
  const actions=node('div',undefined,'device-actions');
  for(const [label,href] of surface==='lab'?[['Open REA',routes.process],['Library',routes.library]]:[['Inspect in Library',routes.library],['Lab status',routes.lab]])actions.append(actionLink(label,href));
  actions.append(actionLink('Timeline',routes.timeline));
  try{const source=new URL(metadata.source_url);if(['https:','http:'].includes(source.protocol)){const link=node('a','Acquisition source');link.href=source.href;link.target='_blank';link.rel='noreferrer';actions.append(link);}}catch{}
  panel.insertBefore(actions,stages);return panel;
}

export function deviceError(error){
  const panel=node('main',undefined,'device-overview');panel.id=document.querySelector('main')?.id||'main-content';panel.tabIndex=-1;panel.append(node('h1','Device unavailable'),node('p',error.message));
  const actions=node('div',undefined,'device-actions'),retry=commandButton('Retry',{kind:'primary'});retry.addEventListener('click',()=>location.reload());actions.append(retry,actionLink('Choose firmware','/library/'));panel.append(actions);document.querySelector('main').replaceWith(panel);
}
