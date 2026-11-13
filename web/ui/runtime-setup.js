import {element as node,commandButton} from './components.js';

export function runtimeSetup({module='Firmware analysis',message='New analysis requires a local workspace and your own analysis provider.'}={}){
  const root=node('details',undefined,'runtime-setup'),summary=node('summary','Set up '+module.toLowerCase()),description=node('p',message),actions=node('div'),copy=commandButton('Copy agent setup',{kind:'secondary',size:'sm'}),status=node('span');
  const request=`Set up ${module.toLowerCase()} in my local Reforge Bench checkout. Inspect its README and existing tools first. Reuse my own compatible analysis provider; do not require the author's personal setup. Identify my OS and install only missing runtimes from official sources. Give me exact commands, preserve existing firmware and sessions, verify the selected image hash/load map, and report what actually runs versus what remains unsupported.`;
  const prompt=node('pre',request);prompt.tabIndex=0;status.setAttribute('role','status');
  copy.addEventListener('click',async()=>{try{await navigator.clipboard.writeText(request);status.textContent='Copied';}catch{status.textContent='Select and copy the setup request below.';prompt.focus();}});
  const install=node('a','Ghidra installation guide');install.href='https://github.com/NationalSecurityAgency/ghidra#install';install.target='_blank';install.rel='noopener noreferrer';
  actions.append(copy,install,status);root.append(summary,description,actions,prompt);return root;
}
