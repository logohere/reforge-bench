import assert from 'node:assert/strict';
import {readBenchState,writeBenchState} from './bench-state.js';
import {benchPlan} from './plan.js';
import {benchLayoutFor} from './layout.js';

let stored=null;
const storage={getItem(){return stored;},setItem(key,value){stored=value;}};
assert.equal(readBenchState(storage,'bench'),null);
stored='{broken';assert.equal(readBenchState(storage,'bench'),null);
stored='{"version":99,"mode":"3d"}';assert.equal(readBenchState(storage,'bench'),null);
const setup={mode:'3d',camera:'wiring',planCamera:'overview',sceneCamera:'wiring',net:'response',phase2:false,hosts:false,isolated:false,guide:true,step:11,labels:true,position:[10,200,300],target:[0,0,0],plan:{base:{x:-100,y:-50,width:200,height:100},view:{x:-60,y:-20,width:100,height:50}}};
writeBenchState(storage,'bench',setup);
assert.deepEqual(readBenchState(storage,'bench'),{...Object.fromEntries(Object.entries(setup).filter(([key])=>!['position','target'].includes(key))),scene:{position:setup.position,target:setup.target}});
stored=JSON.stringify({version:1,mode:'bad',camera:'bad',planCamera:'bad',sceneCamera:'bad',net:'bad',guide:'true',step:100,position:[0,0,0],target:[0,0,0],plan:{base:{x:0,y:0,width:0,height:10},view:{x:0,y:0,width:1,height:1}}});
assert.deepEqual(readBenchState(storage,'bench'),{});
assert.equal(readBenchState({getItem(){throw Error('blocked');}},'bench'),null);
assert.doesNotThrow(()=>writeBenchState({setItem(){throw Error('quota');}},'bench',setup));
stored=JSON.stringify({version:1,mode:'2d',camera:'wiring',net:'pulse'});assert.deepEqual(readBenchState(storage,'bench'),{mode:'2d',camera:'wiring',net:'pulse'});

class Element extends EventTarget{
  constructor(tag){super();this.tagName=tag;this.children=[];this.dataset={};this.attributes={};this.style={setProperty(){}};this.classList={add(){}};}
  append(...children){this.children.push(...children);}
  replaceChildren(...children){this.children=children;}
  setAttribute(name,value){this.attributes[name]=String(value);if(name.startsWith('data-'))this.dataset[name.slice(5).replace(/-([a-z])/g,(_,letter)=>letter.toUpperCase())]=String(value);}
  querySelectorAll(selector){const attribute=selector.slice(1,-1);return this.children.flatMap(child=>[...(child.attributes&&Object.hasOwn(child.attributes,attribute)?[child]:[]),...(child.querySelectorAll?.(selector)||[])]);}
}
globalThis.document={createElement:tag=>new Element(tag),createElementNS:(namespace,tag)=>new Element(tag)};
const plan=benchPlan();
const descendants=element=>element.children.flatMap(child=>[child,...(child.children?descendants(child):[])]);
const coraDetail=descendants(plan.svg).find(child=>child.attributes?.['aria-label']==='Cora board overview and TPH2 pin detail');
plan.update({part:'cora',camera:'wiring',reframe:true});
assert.equal(coraDetail.style.display,'');
assert.ok(descendants(coraDetail).some(child=>child.attributes?.['data-board-model']==='cora-z7-07s'));
assert.ok(!descendants(coraDetail).some(child=>child.tagName==='image'));
assert.equal(descendants(plan.svg).filter(child=>child.attributes?.['data-board-model']==='cora-z7-07s').length,2);
for(const feature of ['tip','ground-collar','finger-guard','attenuation-switch','strain-relief'])assert.ok(descendants(plan.svg).some(child=>child.attributes?.['data-probe-feature']===feature));
assert.equal(descendants(coraDetail).filter(child=>child.attributes?.['data-tph-pin']).length,12);
plan.update({part:'tph',camera:'wiring',reframe:true});
const tphDetail=descendants(plan.svg).find(child=>child.attributes?.['aria-label']==='TPH2 enlarged top view and all 12 pin roles');
assert.equal(tphDetail.style.display,'');
assert.equal(descendants(tphDetail).filter(child=>child.attributes?.['data-tph-pin']).length,12);
plan.update({part:'breadboard',contact:'C55',selected:'pulse',run:0,camera:'wiring',reframe:true});
const contactDetail=descendants(plan.svg).find(child=>child.attributes?.class==='hardware-plan-selected-contacts');
assert.equal(contactDetail.children.filter(child=>child.tagName==='text').length,1,'Inspecting one hole must not stack labels for every connected hole');
assert.equal(contactDetail.children.filter(child=>child.tagName==='circle').length,1);
assert.ok(!descendants(plan.svg).some(child=>child.attributes?.class==='hardware-plan-leader'),'Decorative label leaders must not look like electrical wires');
assert.ok(descendants(plan.svg).some(child=>child.attributes?.['data-instrument-model']==='lonely-binary-analyzer'));
plan.update({part:'scope',camera:'wiring',reframe:true});
const scopeDetail=descendants(plan.svg).find(child=>child.attributes?.['aria-label']==='Hantek front controls and rear USB detail');
assert.equal(scopeDetail.style.display,'');
assert.deepEqual(descendants(scopeDetail).filter(child=>child.attributes?.['data-scope-port']).map(child=>child.attributes['data-scope-port']),['CH1','CH2','EXT / GEN']);
plan.update({selected:'pulse',camera:'wiring',reframe:true});
assert.equal(coraDetail.style.display,'none');
plan.update({selected:'pulse',camera:'wiring',reframe:true});
plan.navigation.zoomBy(1.4);
const view=plan.navigation.state;
for(const state of [{selected:'ground',run:0},{selected:'response',run:3},{selected:'usbCora',hosts:true},{selected:'sda',phase2:true}]){
  plan.update({...state,camera:'wiring'});
  assert.deepEqual(plan.navigation.state,view,'Selection must not change either plan view or its reset target');
}
plan.update({selected:'ground',run:0,camera:'wiring',reframe:true});
assert.notDeepEqual(plan.navigation.state,view,'Explicit Focus must change the view');
plan.navigation.restore(view);assert.deepEqual(plan.navigation.state,view);
plan.dispose();
const ddcsPlan=benchPlan({layout:benchLayoutFor('ddcs')});
ddcsPlan.update({selected:'uartTx',run:0,camera:'wiring',isolated:false,reframe:true});
for(const id of ['uartTx','uartRx']){
  const net=descendants(ddcsPlan.svg).find(child=>child.attributes?.['data-net']===id);
  assert.notEqual(net.style.display,'none');
  assert.ok(descendants(net).some(child=>child.tagName==='path'&&child.attributes?.d?.startsWith('M')),'DDCS UART must have a drawn wire path, not only text');
}
const ddcsContacts=descendants(ddcsPlan.svg).find(child=>child.attributes?.class==='hardware-plan-selected-contacts');
assert.ok(ddcsContacts.children.some(child=>child.tagName==='text'&&child.textContent==='A23'));
assert.ok(ddcsContacts.children.some(child=>child.tagName==='text'&&child.textContent==='TPH2.3'));
ddcsPlan.dispose();
console.log('PASS: setup round-trip, invalid/obsolete/blocked storage, independent plan state, selection stability and explicit framing.');
