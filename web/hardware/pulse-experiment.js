const ids=['nominal','missing','stuck','reset'];
export function validateExperiments(bundle){
  const supportedEngine=bundle?.origin==='Icarus Verilog RTL simulation'&&bundle.tool?.startsWith('Icarus Verilog version ')||bundle?.origin==='AMD Vivado XSim RTL simulation'&&bundle.tool==='AMD Vivado XSim 2025.2';
  if(bundle?.schema!==1||!supportedEngine||bundle.configuration?.simulationClock_Hz!==1000000||bundle.configuration.watchdogCycles!==100000||bundle.configuration.edgesPerToggle!==1000)throw new Error('Unsupported experiment configuration');
  if(JSON.stringify(bundle.configuration.sampleColumns)!==JSON.stringify(['time_ns','stimulus','heartbeat','fault','reset']))throw new Error('Unsupported trace columns');
  for(const path of ['web/hardware/pulse_loop.v','web/hardware/pulse_experiment_tb.v','scripts/build-pulse-experiments.py'])if(!/^[a-f0-9]{64}$/.test(bundle.sourceSha256?.[path]))throw new Error('Missing source identity');
  if(bundle.scenarios?.length!==ids.length)throw new Error('Incomplete experiment set');
  bundle.scenarios.forEach((scenario,index)=>{
    if(scenario.id!==ids[index]||scenario.result?.passed!==true||!scenario.result.assertions?.length||!Array.isArray(scenario.samples)||scenario.samples.length<2||scenario.samples.length>20000||scenario.duration_ns!==2600010000)throw new Error('Invalid experiment result');
    let previous=-1;
    for(const row of scenario.samples){if(!Array.isArray(row)||row.length!==5||!Number.isSafeInteger(row[0])||row[0]<=previous||row[0]>scenario.duration_ns||row.slice(1).some(value=>value!==0&&value!==1))throw new Error('Invalid digital trace');previous=row[0];}
    if(scenario.samples[0][0]!==0||previous!==scenario.duration_ns)throw new Error('Incomplete trace interval');
    const rising=[],toggles=[],faults=[],clearances=[],releases=[];
    for(let i=1;i<scenario.samples.length;i++){const a=scenario.samples[i-1],b=scenario.samples[i];if(!a[1]&&b[1])rising.push(b[0]);if(a[2]!==b[2]&&!b[3]&&!b[4])toggles.push(b[0]);if(!a[3]&&b[3])faults.push(b[0]);if(a[3]&&!b[3])clearances.push(b[0]);if(a[4]&&!b[4])releases.push(b[0]);}
    if(toggles.length!==2||faults.length!==(scenario.id==='nominal'?0:1))throw new Error('Trace contradicts pulse-loop expectations');
    for(let i=0;i<toggles.length;i++){const end=toggles[i],start=Math.max(i?toggles[i-1]:0,...faults.filter(time=>time<end),...releases.filter(time=>time<end));if(rising.filter(time=>time>start&&time<=end).length!==1000)throw new Error('Response did not follow 1,000 new input edges');}
    const measurements=scenario.result.measurements;
    for(const [key,times] of [['healthyResponseTransitions_s',toggles],['faultAssertions_s',faults],['faultClearances_s',clearances]])if(JSON.stringify(measurements?.[key])!==JSON.stringify(times.map(time=>time/1e9)))throw new Error('Result does not describe the loaded trace');
    if(['missing','stuck'].includes(scenario.id)){const lastEdge=rising.filter(time=>time<faults[0]).at(-1),latency=faults[0]-lastEdge;if(latency<100000000||latency>100005000||measurements.watchdogFromStimulusEdge_s!==latency/1e9)throw new Error('Watchdog trace or reported interval differs');}
  });
  return bundle;
}

export function experimentSession(bundle,traceSha256){
  bundle=structuredClone(validateExperiments(bundle));
  function freeze(value){if(value&&typeof value==='object'){for(const child of Object.values(value))freeze(child);Object.freeze(value);}}freeze(bundle);
  if(!/^[a-f0-9]{64}$/.test(traceSha256))throw new Error('Missing trace identity');
  let scenario=bundle.scenarios[0],state='ready',position=0;
  return {
    get scenario(){return scenario;},get state(){return state;},get position(){return position;},
    select(id){if(state==='running')return false;const next=bundle.scenarios.find(item=>item.id===id);if(!next)throw new Error('Unknown experiment');scenario=next;this.reset();return true;},
    run(){if(!['ready','held'].includes(state)||position>=scenario.duration_ns)return false;state='running';return true;},
    hold(){if(state!=='running')return false;state='held';return true;},
    reset(){position=0;state='ready';},
    advance(seconds){if(!Number.isFinite(seconds)||seconds<0)throw new Error('Invalid replay interval');if(state==='running'){position=Math.min(scenario.duration_ns,position+seconds*1e9);if(position===scenario.duration_ns)state='complete';}},
    seek(seconds){if(!Number.isFinite(seconds)||seconds<0||seconds>scenario.duration_ns/1e9)throw new Error('Invalid replay position');position=seconds*1e9;state=position===scenario.duration_ns?'complete':position===0?'ready':'held';},
    snapshot(){let low=0,high=scenario.samples.length;while(low+1<high){const mid=(low+high)>>1;if(scenario.samples[mid][0]<=position)low=mid;else high=mid;}return scenario.samples[low];},
    receipt(){return {schema:1,origin:bundle.origin,scope:bundle.scope,tool:bundle.tool,traceSha256,sourceSha256:bundle.sourceSha256,configuration:bundle.configuration,scenario:scenario.id,replay:{state,position_s:position/1e9,completed:state==='complete'},result:state==='complete'?scenario.result:null,qualified:false};}
  };
}
