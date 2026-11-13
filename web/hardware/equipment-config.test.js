import assert from 'node:assert/strict';
import {equipmentExample, parseEquipment} from './equipment-config.js';

assert.deepEqual(parseEquipment(JSON.stringify(equipmentExample)), equipmentExample);
const custom = {schema: 'reforge-equipment/1', name: 'Custom bench', devices: [
  {id: 'target', name: 'User controller', role: 'Target', interface: 'UART'},
  {id: 'capture', name: 'Other instrument', role: 'Acquisition', interface: 'Ethernet'}
]};
assert.deepEqual(parseEquipment(JSON.stringify(custom)), custom);
assert.deepEqual(parseEquipment(JSON.stringify({...custom, devices: []})).devices, []);
const withPrivateExtras = {...custom, account: 'not-exported', devices: [{...custom.devices[0], password: 'not-exported'}]};
assert.equal(JSON.stringify(parseEquipment(JSON.stringify(withPrivateExtras))).includes('not-exported'), false);
for (const value of [null, {}, {...custom, schema: 'future'}, {...custom, devices: [custom.devices[0], custom.devices[0]]},
  {...custom, devices: [null]}, {...custom, name: '\n'}, {...custom, devices: Array(65).fill(custom.devices[0])}]) {
  assert.throws(() => parseEquipment(JSON.stringify(value)));
}
assert.throws(() => parseEquipment(' '.repeat(65537)));
assert.throws(() => parseEquipment('{broken'));
console.log('PASS: interchangeable equipment records, example round-trip, import limits and allowlisted exports.');
