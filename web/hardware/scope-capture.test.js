import assert from 'node:assert/strict';
import {parseScopeCapture,comparePulseCapture} from './scope-capture.js';

const time=Array.from({length:5001},(_,i)=>i*1e-6),voltage=time.map((_,i)=>i%1000<500?3.3:0),csv='Time(s),Voltage(V)\n'+time.map((t,i)=>t+','+voltage[i]).join('\n');
const capture=parseScopeCapture(csv);assert.equal(capture.origin,'import');assert.ok(Math.abs(capture.frequency-1000)<1e-8);assert.equal(comparePulseCapture(capture).status,'within demo limits');
const replay=parseScopeCapture(JSON.stringify({schema:'rea-scope-trace/1',origin:'simulation',time,voltage}));assert.equal(replay.origin,'simulation');assert.equal(replay.source,'CH1 · simulation replay');
for(const phase of [0,137,499,731]){
  const partialTime=Array.from({length:2701},(_,i)=>i*1e-6),partialVoltage=partialTime.map((_,i)=>(i+phase)%1000<500?3.3:0);
  const partial=parseScopeCapture(JSON.stringify({schema:'rea-scope-trace/1',origin:'simulation',time:partialTime,voltage:partialVoltage}));
  assert.ok(Math.abs(partial.duty-.5)<1e-9,'Partial cycles must not bias duty');
  assert.equal(comparePulseCapture(partial).status,partial.completeCycles>=2?'within demo limits':'insufficient cycles');
}
assert.equal(parseScopeCapture('nr,time,ch1,ch2,ch1s,ch2s\n0,0,0,0,0,0\n1,0.001,3.3,0,1,0\n2,0.002,0,0,0,0').format,'phmarek SCPI CSV/TSV');
assert.equal(comparePulseCapture(parseScopeCapture('Time(s),Voltage(V)\n0,0\n0.001,0\n0.002,0')).status,'outside demo limits');
assert.equal(comparePulseCapture(parseScopeCapture(csv.replaceAll(',3.3',',33'))).status,'outside demo limits');
for(const text of ['Time,Voltage\n0,0\n1,1\n2,0','Time(s),Voltage(V)\n0,0\n0,1\n1,0','Time(s),Voltage(V)\n0,0\n1,NaN\n2,0','Time(s),Voltage(V)\n0,0\n1,\n2,0'])assert.throws(()=>parseScopeCapture(text));
console.log('PASS: real waveform parsing, explicit units, simulation provenance, pulse comparison and invalid-record rejection.');
