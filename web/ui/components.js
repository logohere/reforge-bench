import {mountSimulationStop} from '../simulation-stop.js';
import {showVersion,showBuild} from '../version.js';
if(typeof window!=='undefined')await import('/dist/ui-components.js');
export function element(tag,text,className){
  const node=document.createElement(tag);
  if(text!==undefined)node.textContent=text;
  if(className)node.className=className;
  return node;
}

export function badge(text,tone='neutral'){
  return element('span',text,'ui-badge ui-badge--'+tone);
}

export function labeledControl(text,control){
  const label=element('label',undefined,'ui-field');label.append(element('span',text,'ui-field-label'),control);return label;
}

export function actionLink(text,href,{primary=false,download}={}){
  const link=commandButton(text,{kind:primary?'primary':'tertiary'});link.href=href;link.linkRole='link';if(download!==undefined)link.download=download;return link;
}

export function commandButton(text,{kind='tertiary',size='md'}={}){
  const button=element('cds-button',text);button.kind=kind;button.size=size;button.type='button';return button;
}

export function setPressed(button,value){
  button.dataset.pressed=String(value);button.removeAttribute('aria-pressed');
  void button.updateComplete.then(()=>button.shadowRoot.querySelector('button')?.setAttribute('aria-pressed',button.dataset.pressed));
}

export function modalDialog({title='Details'}={}){
  const root=element('dialog',undefined,'ui-dialog'),header=element('div',undefined,'ui-dialog-header'),heading=element('h2',title),body=element('div',undefined,'ui-dialog-body'),close=commandButton('Close',{kind:'ghost',size:'sm'});
  modalDialog.sequence=(modalDialog.sequence||0)+1;const id='ui-dialog-'+modalDialog.sequence;heading.id=id;body.tabIndex=-1;root.setAttribute('aria-labelledby',id);header.append(heading,close);root.append(header,body);let opener=null,outside=false;
  const isOutside=event=>{const box=root.getBoundingClientRect();return event.clientX<box.left||event.clientX>box.right||event.clientY<box.top||event.clientY>box.bottom;};
  root.addEventListener('pointerdown',event=>{outside=event.target===root&&isOutside(event);});
  root.addEventListener('click',event=>{if(outside&&event.target===root&&isOutside(event))root.close();outside=false;});
  root.addEventListener('keydown',event=>{if(event.key==='Escape'){event.preventDefault();root.close();}});close.addEventListener('click',()=>root.close());root.addEventListener('close',()=>{outside=false;if(opener?.isConnected)opener.focus();});
  function show(nextTitle,content,returnFocus=document.activeElement){heading.textContent=nextTitle;body.replaceChildren(...(Array.isArray(content)?content:[content]));if(!root.open)opener=returnFocus;outside=false;if(!root.open)root.showModal();close.focus();}
  return {root,body,heading,show,close:()=>root.close()};
}

export function choiceField({id,label,items,value=items[0]?.[0]}){
  const input=element('cds-select');input.id=id;input.labelText=label;input.size='md';input.value=String(value);
  for(const [key,text] of items){const item=element('cds-select-item',text);item.setAttribute('value',String(key));if(String(key)===String(value))item.setAttribute('selected','');input.append(item);}
  return {root:input,input};
}

export function viewSwitcher({label,items,value=items[0]?.[0],onChange=()=>{}}){
  const root=element('cds-content-switcher');root.setAttribute('role','tablist');root.setAttribute('aria-label',label);root.size='md';root.value=String(value);
  for(const [id,text] of items){const item=element('cds-content-switcher-item',text);item.value=String(id);item.setAttribute('value',String(id));root.append(item);}
  root.addEventListener('cds-content-switcher-selected',event=>onChange(event.detail.item.value));return root;
}

export function radioChoices({id,label,items,value=items[0]?.[0],orientation='vertical'}){
  const input=element('cds-radio-button-group');input.id=id;input.name=id;input.legendText=label;input.value=String(value);input.orientation=orientation;
  for(const [key,text] of items){const item=element('cds-radio-button');item.value=String(key);item.labelText=text;item.name=id;item.checked=String(key)===String(value);input.append(item);}
  return {root:input,input};
}

export function callout(title,message,tone='info'){
  const root=element('aside',undefined,'ui-callout');root.dataset.tone=tone;root.append(element('strong',title),element('p',message));return root;
}

export function quantityField({id,label,unit='',value,min=-Infinity,max=Infinity,hint=''}){
  if(!id||!label||Number.isNaN(min)||Number.isNaN(max)||min>max)throw new Error('Invalid quantity field');
  const input=element('cds-text-input'),root=input;input.id=id;input.label=label+(unit?' · '+unit:'');input.size='md';input.value=String(value);input.required=true;input.helperText=hint;
  async function syncInput(){await input.updateComplete;const control=input.shadowRoot.querySelector('input');control.inputMode='decimal';control.setAttribute('aria-invalid',String(input.invalid));}
  void syncInput();
  function validate(){
    const text=input.value.trim(),number=Number(text),valid=/^[+-]?(?:\d+(?:\.\d*)?|\.\d+)(?:e[+-]?\d+)?$/i.test(text)&&Number.isFinite(number)&&number>=min&&number<=max;
    const range=Number.isFinite(min)&&Number.isFinite(max)?` from ${min} to ${max}`:Number.isFinite(min)?` of at least ${min}`:Number.isFinite(max)?` no greater than ${max}`:'';
    const error=valid?'':`Enter a number${range}${unit?' '+unit:''}.`;input.invalid=!valid;input.invalidText=error;void syncInput();return valid;
  }
  input.addEventListener('focusout',validate);input.addEventListener('input',()=>{if(input.invalid)validate();});
  return {root,input,validate,get value(){return Number(input.value);}};
}

export function propertyList(rows){
  const list=element('dl',undefined,'ui-property-list');for(const [label,value] of rows){const row=element('div');row.append(element('dt',label));const content=element('dd');content.append(value instanceof Node?value:document.createTextNode(String(value)));row.append(content);list.append(row);}return list;
}

export function evidenceStages(rows){
  const list=element('ol',undefined,'ui-evidence-stages');list.setAttribute('aria-label','Evidence stages');
  for(const [label,value,tone='neutral'] of rows){const stage=element('li');stage.append(element('strong',label),badge(value,({muted:'neutral',pass:'success',warn:'warning',info:'info'})[tone]||tone));list.append(stage);}
  return list;
}

export function metric(label,value,unit=''){
  const item=element('div',undefined,'ui-metric'),term=element('dt',label);
  if(unit)term.append(element('span',unit,'ui-metric-unit'));item.append(term,value);return item;
}

export function mountTabs(root,entries,{id,label,variant='line',fullWidth=false,preservePanelIds=false,value=entries[0]?.id,onChange=()=>{}}){
  if(!entries.length||new Set(entries.map(entry=>entry.id)).size!==entries.length)throw new Error('Tabs require unique entries');
  root.classList.add('ui-tabs','ui-tabs--'+variant);const tabs=element('cds-tabs');tabs.setAttribute('aria-label',label);tabs.fullWidth=fullWidth;tabs.type=variant==='contained'?'contained':'';tabs.size='md';root.append(tabs);const buttons={};
  function select(value,focus=false){
    const selected=entries.find(entry=>entry.id===value);if(!selected)throw new Error('Unknown tab');
    tabs.value=value;for(const entry of entries){const active=entry===selected;entry.panel.hidden=!active;buttons[entry.id].selected=active;}
    selected.panel.parentElement?.scrollTo({top:0});if(focus)buttons[value].focus();onChange(value);
  }
  for(const entry of entries){
    const button=element('cds-tab',entry.label);button.id=entry.tabId||`${id}-tab-${entry.id}`;button.value=entry.id;if(!preservePanelIds)entry.panel.id=`${id}-panel-${entry.id}`;button.target=entry.panel.id;entry.panel.setAttribute('role','tabpanel');entry.panel.setAttribute('aria-labelledby',button.id);entry.panel.tabIndex=0;
    buttons[entry.id]=button;tabs.append(button);
    void button.updateComplete.then(()=>button.shadowRoot.querySelector('[role=tab]')?.setAttribute('aria-controls',entry.panel.id));
  }
  tabs.addEventListener('cds-tabs-selected',event=>select(event.detail.item.value));
  const observer=new ResizeObserver(()=>{if(!root.isConnected){observer.disconnect();return;}const width=tabs.getBoundingClientRect().width,total=Object.values(buttons).reduce((sum,button)=>sum+button.getBoundingClientRect().width,0);if(width&&total)tabs.dataset.fits=String(total+2<=width);});observer.observe(root);
  select(value);return {select,buttons};
}

export function icon(name){
  if(!['chevron--down','restart','play','pause','arrow--left','arrow--right','upload','catalog','tools','flow','time','help','side-panel--close','side-panel--open'].includes(name))throw new Error('Unknown bench icon');
  const node=element('span',undefined,'ui-icon');node.setAttribute('aria-hidden','true');node.style.setProperty('--icon',`url("/ui/icons/${name}.svg")`);return node;
}

export function actionLabel(control,text,name){
  control.classList.add('button-leading');const content=element('span',undefined,'button-content');content.append(icon(name),element('span',text));control.replaceChildren(content);
}

let shellObserver;
export function observeShell(){
  const header=document.querySelector('body>header'),context=document.querySelector('.device-context');
  if(!header)return;
  shellObserver?.disconnect();
  const measure=()=>{const height=header.getBoundingClientRect().height;document.body.style.setProperty('--bench-header-height',height+'px');document.body.style.setProperty('--shell-height',(height+(context&&!header.contains(context)?context.getBoundingClientRect().height:0))+'px');};
  shellObserver=new ResizeObserver(measure);shellObserver.observe(header);if(context)shellObserver.observe(context);measure();
}

export function mountShell({hardwareOnly=false}={}){
  const header=document.querySelector('body>header');
  if(!header||header.dataset.component==='shell')return;
  header.dataset.component='shell';header.className='app-header';
  document.body.dataset.shell='sidebar';
  if(window.__REA_RUNE_COMPAT__)document.body.dataset.host='rune';
  let collapsed=hardwareOnly;try{const saved=localStorage.getItem('rea.navigation.collapsed');if(saved!==null)collapsed=saved==='true';}catch{}
  const shell=element('cds-header',undefined,'bench-header'),menu=element('cds-header-menu-button');menu.buttonLabelInactive='Open workspace navigation';menu.buttonLabelActive='Close workspace navigation';
  const brand=element('a',undefined,'brand');
  const version=element('small',undefined,'app-version');version.hidden=true;
  const brandIcon=element('img');brandIcon.src='/assets/brand/rea-app.png';brandIcon.alt='REA';brandIcon.width=32;brandIcon.height=32;
  brand.append(brandIcon,element('span','Reforge Bench'));void showVersion(version);
  let caseId=new URLSearchParams(location.search).get('case');try{caseId||=sessionStorage.getItem('rea.active-device.v1');}catch{}
  if(!/^[a-f0-9]{64}$/.test(caseId||''))caseId=null;
  brand.href=hardwareOnly?'/hardware/':'/home/'+(caseId?'?case='+caseId:'');brand.title=hardwareOnly?'Hardware bench':'Bench home';
  if(document.body.dataset.workspace==='home')brand.setAttribute('aria-current','page');
  const sidebar=element('div',undefined,'workspace-sidebar'),nav=element('nav',undefined,'workspace-tabs'),items=element('cds-side-nav-items');nav.id='workspace-navigation';nav.setAttribute('aria-label','Workspace');menu.setAttribute('aria-controls',nav.id);
  for(const [id,label,href,name] of [['library','Library','/library/','catalog'],['lab','Lab','/lab/','tools'],['rea','REA Tool','/rea/#target','flow'],['timeline','Timeline','/timeline/','time'],['hardware','Hardware','/hardware/','tools']]){
    if(hardwareOnly&&id!=='hardware')continue;
    const link=element('cds-side-nav-link',label),symbol=icon(name);symbol.slot='title-icon';link.append(symbol);link.large=true;const route=new URL(href,location.origin);if(caseId&&!hardwareOnly)route.searchParams.set('case',caseId);link.href=route.pathname+route.search+route.hash;link.title=label;link.dataset.surface=id;items.append(link);
  }
  const toggle=commandButton('',{kind:'ghost',size:'sm'}),toggleLabel=element('span',undefined,'visually-hidden'),heading=element('div',undefined,'sidebar-heading');toggle.classList.add('navigation-toggle');toggle.setAttribute('aria-controls',nav.id);
  function setCollapsed(value){collapsed=value;document.body.classList.toggle('navigation-collapsed',value);toggleLabel.textContent=value?'Expand navigation':'Collapse navigation';toggle.title=toggleLabel.textContent;toggle.setAttribute('aria-expanded',String(!value));toggle.replaceChildren(icon(value?'side-panel--open':'side-panel--close'),toggleLabel);void toggle.updateComplete.then(()=>{const control=toggle.shadowRoot.querySelector('button');control?.setAttribute('aria-expanded',String(!collapsed));control?.setAttribute('aria-controls',nav.id);});try{localStorage.setItem('rea.navigation.collapsed',String(value));}catch{}}
  toggle.addEventListener('click',()=>setCollapsed(!collapsed));setCollapsed(collapsed);
  heading.append(element('span','Workspace'),toggle);nav.append(items);sidebar.append(heading,nav);
  if(document.body.dataset.workspace==='process'){
    const investigation=document.querySelector('main>aside');
    if(investigation)investigation.className='process-navigation';
  }
  const help=element('a',undefined,'shell-help');help.append(icon('help'),element('span','Help'));help.href='/help/'+(caseId&&!hardwareOnly?'?case='+caseId:'');help.title='Help';help.hidden=hardwareOnly;if(document.body.dataset.workspace==='help')help.setAttribute('aria-current','page');
  const build=element('small');build.hidden=true;void showBuild(build);
  const guides=element('a',undefined,'shell-help shell-guides');guides.append(icon('help'),element('span','Setup guides'));guides.href='/guides/';guides.title='Setup guides';if(document.body.dataset.workspace==='guides')guides.setAttribute('aria-current','page');
  const foot=element('div',undefined,'sidebar-footer');foot.append(guides,help,version,build);sidebar.append(foot);
  const context=element('div',undefined,'shell-context-slot'),actions=element('div',undefined,'shell-actions');if(document.body.dataset.workspace==='lab')mountSimulationStop(actions);
  shell.append(menu,brand,context,actions);header.replaceChildren(shell);header.after(sidebar);
  menu.addEventListener('cds-header-menu-button-toggled',event=>{document.body.classList.toggle('navigation-open',event.detail.active);});
  document.addEventListener('keydown',event=>{if(event.key==='Escape'&&document.body.classList.contains('navigation-open')){document.body.classList.remove('navigation-open');menu.active=false;menu.focus();}});
  document.addEventListener('click',event=>{if(document.body.classList.contains('navigation-open')&&!sidebar.contains(event.target)&&!menu.contains(event.target)){document.body.classList.remove('navigation-open');menu.active=false;}});
  const main=document.querySelector('main');
  if(main&&!document.querySelector('.skip-link')){if(!main.id)main.id='main-content';main.tabIndex=-1;const skip=element('a','Skip to content','skip-link');skip.href='#'+main.id;document.body.prepend(skip);}
  observeShell();
}
