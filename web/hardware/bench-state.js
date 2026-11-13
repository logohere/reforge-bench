import {benchLayout} from './layout.js';
import {wiringStepsFor} from './wiring-guide.js';

const vector=value=>Array.isArray(value)&&value.length===3&&value.every(number=>Number.isFinite(number)&&Math.abs(number)<=10000);
const rectangle=value=>value&&['x','y','width','height'].every(key=>Number.isFinite(value[key])&&Math.abs(value[key])<=10000)&&value.width>0&&value.height>0;

export function readBenchState(storage,key){
  try{
    const value=JSON.parse(storage.getItem(key));
    if(value?.version!==1)return null;
    const state={};
    if(['pulse','ddcs'].includes(value.profile))state.profile=value.profile;
    if(['2d','3d'].includes(value.mode))state.mode=value.mode;
    for(const field of ['camera','planCamera','sceneCamera'])if(['overview','wiring','top'].includes(value[field]))state[field]=value[field];
    if(Object.hasOwn(benchLayout.nets,value.net))state.net=value.net;
    for(const field of ['phase2','hosts','isolated','guide','labels'])if(typeof value[field]==='boolean')state[field]=value[field];
    if(Number.isInteger(value.step)&&value.step>=0&&value.step<wiringStepsFor(state.profile).length)state.step=value.step;
    if(vector(value.position)&&vector(value.target)&&Math.hypot(...value.position.map((number,index)=>number-value.target[index]))>=8)state.scene={position:value.position,target:value.target};
    if(rectangle(value.plan?.base)&&rectangle(value.plan?.view))state.plan=value.plan;
    return state;
  }catch{return null;}
}

export function writeBenchState(storage,key,state){
  try{storage.setItem(key,JSON.stringify({version:1,...state}));}catch{}
}
