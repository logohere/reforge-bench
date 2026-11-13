let handler;
export function registerSimulationStop(stop){handler=stop;const button=document.getElementById('simulation-estop');if(button)button.disabled=false;window.dispatchEvent(new Event('rea-simulation-stop-state'));}
export function mountSimulationStop(parent){
  const button=document.createElement('cds-button');button.id='simulation-estop';button.type='button';button.kind='danger-tertiary';button.size='md';button.className='button-danger simulation-estop';button.textContent='E-stop simulation';button.disabled=true;button.title='Stop this simulated controller or playback. Not a physical machine safety circuit.';
  button.addEventListener('click',async()=>{
    if(!handler)return;button.disabled=true;button.setAttribute('aria-busy','true');window.dispatchEvent(new Event('rea-simulation-stop-state'));
    try{await handler();}
    catch(error){const status=document.getElementById('message');if(status){status.textContent='Stop failed: '+error.message;status.classList.add('error');}}
    finally{button.disabled=false;button.removeAttribute('aria-busy');window.dispatchEvent(new Event('rea-simulation-stop-state'));}
  });parent.append(button);
}
