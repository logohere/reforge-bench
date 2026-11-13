import assert from 'node:assert/strict';
import {benchLayout,benchLayoutFor,benchNetVisible} from './layout.js';
import {checkBenchConnections,analyzerHeader} from './circuit.js';
import {breadboardConnection,railContacts} from './breadboard.js';

assert.deepEqual(analyzerHeader,[['CH0','CH1'],['CH2','CH3'],['CH4','CH5'],['CH6','CH7'],['CLK','GND']]);
assert.deepEqual(checkBenchConnections(benchLayout).errors,[]);
for(const [net,index] of [['pulse',3],['response',1],['ground',3]]){
  const disconnected=structuredClone(benchLayout);disconnected.nets[net].runs.splice(index,1);
  assert.equal(checkBenchConnections(disconnected).ok,false,'Missing analyzer '+net+' lead must fail');
}
const shorted=structuredClone(benchLayout);shorted.nets.pulse.runs.push({from:'A55',to:'B45',points:[[0,13,0],[1,13,0]],kind:'jumper',jacket:'yellow'});
assert.ok(checkBenchConnections(shorted).errors.some(error=>error.startsWith('Short:')));
const crossed=structuredClone(benchLayout);crossed.nets.pulse.runs[3].to='Analyzer.CH1';crossed.nets.response.runs[1].to='Analyzer.CH0';
assert.equal(checkBenchConnections(crossed).ok,false,'Swapped channels must fail');
const powered=structuredClone(benchLayout);powered.nets.ground.runs[1].to='TPH2.6';
assert.ok(checkBenchConnections(powered).errors.includes('Independent power outputs must not be joined'));
for(const contact of ['A26','C26']){
  const occupied=structuredClone(benchLayout);occupied.nets.pulse.runs.push({from:contact,to:'Analyzer.CH2',points:[[0,13,0],[1,13,0]],kind:'jumper',jacket:'yellow'});
  assert.ok(checkBenchConnections(occupied).errors.includes('Contact reused: '+contact));
}
for(const [contact,supply] of [['J27','3V3_OUT'],['J23','VBUS'],['J24','VSYS']]){
  const backfeed=structuredClone(benchLayout);backfeed.nets.ground.runs.push({from:contact,to:'TPH2.5',points:[[0,13,0],[1,13,0]],kind:'jumper',jacket:'black'});
  assert.ok(checkBenchConnections(backfeed).errors.includes('Supply connected to ground: Pico.'+supply));
}
const otherGround=structuredClone(benchLayout);otherGround.nets.ground.runs[0].from='A30';
assert.equal(checkBenchConnections(otherGround).ok,true,'Pico physical pin 8 shares the board ground');
assert.deepEqual(breadboardConnection('C26').contacts.map(point=>point.id),['A26','B26','C26','D26','E26']);
assert.ok(!breadboardConnection('F26').contacts.some(point=>point.id==='E26'));
assert.ok(!breadboardConnection('E26').contacts.some(point=>point.id==='E27'));
for(const point of railContacts){assert.equal(breadboardConnection(point.id).verified,false);assert.equal(breadboardConnection(point.id).contacts.length,1);assert.equal(breadboardConnection(point.id,{rails:'continuous'}).contacts.length,50);assert.equal(breadboardConnection(point.id,{rails:'split'}).contacts.length,25);}
assert.ok(!breadboardConnection('upper-0-25',{rails:'split'}).contacts.some(point=>point.id==='upper-0-26'));
assert.ok(!breadboardConnection('upper-0-1',{rails:'continuous'}).contacts.some(point=>point.id.startsWith('upper-1-')||point.id.startsWith('lower-')));
const railBridge=structuredClone(benchLayout);railBridge.nets.pulse.runs.push({from:'A55',to:'upper-0-1',points:[[0,13,0],[1,13,0]],kind:'jumper',jacket:'yellow'});railBridge.nets.ground.runs.push({from:'B45',to:'upper-0-50',points:[[0,13,0],[1,13,0]],kind:'jumper',jacket:'black'});
assert.ok(checkBenchConnections(railBridge).errors.some(error=>error.startsWith('Rail continuity unverified:')));
assert.ok(checkBenchConnections(railBridge,{rails:'continuous'}).errors.some(error=>error.startsWith('Short:')));
assert.ok(!checkBenchConnections(railBridge,{rails:'split'}).errors.some(error=>error.startsWith('Short:')));
console.log('PASS: pulse/return/common-ground paths; missing grounds, swapped channels, shorts, occupied holes and supply backfeed rejected.');

const ddcs=benchLayoutFor('ddcs');
assert.equal(checkBenchConnections(ddcs).ok,true);
assert.equal(ddcs.nets.pulse.label,'STEP');assert.equal(ddcs.nets.response.label,'DIR');
for(const id of ['uartTx','uartRx']){
  assert.equal(benchNetVisible(ddcs.nets[id],{profile:'ddcs'}),true);
  assert.equal(benchNetVisible(ddcs.nets[id],{profile:'pulse'}),false);
  const disconnected=structuredClone(ddcs);disconnected.nets[id].runs=[];
  assert.equal(checkBenchConnections(disconnected).ok,false);
}
const uartSwapped=structuredClone(ddcs);uartSwapped.nets.uartTx.runs[0].to='TPH2.4';uartSwapped.nets.uartRx.runs[0].to='TPH2.3';assert.equal(checkBenchConnections(uartSwapped).ok,false);
const uartShort=structuredClone(ddcs);uartShort.nets.uartTx.runs[0].to='TPH2.1';assert.ok(checkBenchConnections(uartShort).errors.some(error=>error.startsWith('Short:')));
assert.equal(benchLayout.nets.pulse.label,'Pulse','DDCS role changes must not mutate the original pulse profile');
console.log('PASS: DDCS UART topology, direction/STEP labels, profile visibility, missing/swapped UART and cross-net short rejection; original profile preserved.');
