import {test,expect} from 'bun:test';
import {readFileSync} from 'node:fs';
import {coraJaPins} from './cora-board.js';
import {pulseLoopContacts,picoPins,sameTerminalStrip} from './breadboard.js';
const root=new URL('../../',import.meta.url);
test('Cora pin detail agrees with pinned Digilent Rev B source and shared wiring contacts',()=>{
 const xml=readFileSync(new URL('hardware/cora/boards/cora-z7-07s/B.0/part0_pins.xml',root),'utf8');
 expect(coraJaPins.map(pin=>pin.pin)).toEqual(Array.from({length:12},(_,i)=>i+1));
 for(const pin of coraJaPins.filter(pin=>pin.package))expect(xml).toMatch(new RegExp('name\\s*=\\s*"JA'+pin.pin+'"[^>]*loc="'+pin.package+'"'));
 expect(coraJaPins.filter(pin=>pin.label==='GND').map(pin=>pin.pin)).toEqual([5,11]);
 expect(coraJaPins.filter(pin=>pin.label==='VCC').map(pin=>pin.pin)).toEqual([6,12]);
 expect(sameTerminalStrip(picoPins[2].contact,pulseLoopContacts.ground.source)).toBe(true);
 expect(sameTerminalStrip(picoPins[3].contact,pulseLoopContacts.pulse.source)).toBe(true);
});
