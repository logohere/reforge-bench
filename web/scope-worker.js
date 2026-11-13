import {Simulation} from 'eecircuit-engine';
import {scopeNetlist,readScopeResult} from './hardware/scope-model.js';

let simulation;
const reportError=error=>self.postMessage({ok:false,error:'Circuit engine: '+String(error?.message||error).replace(/data:[^\s\x22\x27)]+/g,'embedded asset').slice(0,240)});
self.addEventListener('unhandledrejection',event=>{event.preventDefault();reportError(event.reason);});
self.onmessage=async({data})=>{
  try{self.postMessage({phase:'Starting circuit engine…'});simulation??=new Simulation();await simulation.start();self.postMessage({phase:'Computing waveform…'});simulation.setNetList(scopeNetlist(data));const raw=await simulation.runSim();const errors=simulation.getError();if(errors.length)throw new Error(errors.join('\n'));self.postMessage({ok:true,result:readScopeResult(raw,data)});}catch(error){reportError(error);}
};
