import assert from 'node:assert/strict';
import {mountHantekScope3D} from './hantek-scope3d.js';

class Root{
  constructor(){this.children=[];this.dataset={};this.hidden=false;}
  append(...items){this.children.push(...items);}
  replaceChildren(...items){this.children=items;}
}
let options,disposed=0,resolveReady;
const calls=[],bench={root:new Root(),ready:new Promise(resolve=>{resolveReady=resolve;}),inspectPart(id){calls.push(['inspect',id]);},setMode(mode){calls.push(['mode',mode]);},dispose(){disposed++;}};
const view=mountHantekScope3D({createRoot:()=>new Root(),createStatus:()=>new Root(),loadBench:async()=>({mountHardwareBench(value){options=value;return bench;}})});
await Promise.resolve();
assert.deepEqual(options,{mode:'3d',camera:'wiring',guide:false,isolated:true});
assert.equal(view.root.children[0],bench.root,'The original bench component must be embedded, not a second model');
assert.deepEqual(calls,[],'Scope framing must wait for the original graphics initialization');
view.setActive(false);resolveReady();await view.ready;
assert.deepEqual(calls,[['inspect','scope'],['mode','3d']]);assert.equal(view.root.hidden,true);
view.setActive(true);assert.equal(view.root.hidden,false);
view.dispose();view.dispose();assert.equal(disposed,1);

let created=0,resolveImport;
const late=mountHantekScope3D({createRoot:()=>new Root(),createStatus:()=>new Root(),loadBench:()=>new Promise(resolve=>{resolveImport=resolve;})});
late.dispose();resolveImport({mountHardwareBench(){created++;return bench;}});await late.ready;
assert.equal(created,0,'Disposal before import must not start a renderer or worker');
console.log('PASS: original scope component reuse, ready-before-framing, activation and renderer lifecycle. Native rendering remains a separate acceptance gate.');
