import assert from 'node:assert/strict';
import {scopeAutoScale,scopeOffScale,scopeDivision} from './scope-display.js';
for(const [min,max] of [[12,14],[-14,-12],[-3.3,3.3],[24,24]]){
  const result={time:[12,12.001,12.002],min,max},scale=scopeAutoScale(result);
  assert.ok(Math.abs(scale.timebase-.0002)<1e-12);assert.equal(scopeOffScale({result,...scale}),false);
  const y=value=>7-(value-scale.offsetVolts)/scale.volts;
  assert.ok(y(max)>=1-1e-12&&y(min)<=7+1e-12);
  if(min===max)assert.equal(y(min),4);
}
assert.equal(scopeOffScale({result:{min:0,max:33},volts:1,offsetVolts:0}),true);
assert.equal(scopeOffScale({result:{min:-2,max:3},volts:1,offsetVolts:0}),true);
assert.deepEqual(scopeDivision(.0002,'s'),{value:'200',unit:'µs'});
assert.deepEqual(scopeDivision(.4,'s'),{value:'400',unit:'ms'});
assert.deepEqual(scopeDivision(.55,'V'),{value:'550',unit:'mV'});
for(const result of [{time:[0,0],min:0,max:1},{time:[0,1],min:3,max:1},{time:[0,1],min:0,max:Infinity}])assert.throws(()=>scopeAutoScale(result));
console.log('Display fitting preserves positive/negative DC offsets, centers constant signals and detects overrange; SI divisions and invalid ranges checked.');
