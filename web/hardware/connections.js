import {element as node} from '../ui/components.js';

const equipment=[
  ['pico','Pico 2 · RP2350','GP2 · pulse output'],
  ['tph','Pmod TPH2','Pin 1 in · pin 2 out'],
  ['cora','Cora Z7-07S · FPGA','Count 1,000 edges → toggle'],
  ['sensor','AS5600 · phase 2','One sensor + rotating magnet'],
  ['scope','Hantek DSO2D15','Voltage · frequency · duty'],
  ['analyzer','Logic analyzer + breakout','Edges · pulse count · I²C']
];

export function signalMap({labels={}}={}){
  const root=node('div',undefined,'hardware-signal-map'),svg=document.createElementNS('http://www.w3.org/2000/svg','svg'),cards={};
  root.setAttribute('role','group');root.setAttribute('aria-label','Pulse-loop signal architecture');svg.setAttribute('aria-hidden','true');svg.classList.add('hardware-signal-connectors');root.append(svg);
  for(const [id,title,description] of equipment){const card=node('section',undefined,'hardware-signal-node');card.dataset.device=id;card.append(node('h3',labels[id]?.title||title),node('p',labels[id]?.description||description));cards[id]=card;root.append(card);}
  function route(){const bounds=root.getBoundingClientRect();if(!bounds.width||!bounds.height)return;svg.setAttribute('viewBox',`0 0 ${bounds.width} ${bounds.height}`);svg.replaceChildren();const ns=svg.namespaceURI;
    const port=(id,side,offset=.5)=>{const rect=cards[id].getBoundingClientRect();return side==='left'||side==='right'?[rect[side]-bounds.left,rect.top-bounds.top+rect.height*offset]:[rect.left-bounds.left+rect.width*offset,rect[side]-bounds.top];};
    const path=(points,optional=false,arrow=false)=>{const wire=document.createElementNS(ns,'path');wire.setAttribute('d',points.map(([x,y],i)=>(i?'L':'M')+x+' '+y).join(' '));if(optional)wire.setAttribute('stroke-dasharray','5 4');svg.append(wire);if(arrow){const tip=points.at(-1),from=points.at(-2),angle=Math.atan2(tip[1]-from[1],tip[0]-from[0]),head=document.createElementNS(ns,'path');head.setAttribute('d',`M${tip[0]-8*Math.cos(angle-.45)} ${tip[1]-8*Math.sin(angle-.45)}L${tip[0]} ${tip[1]}L${tip[0]-8*Math.cos(angle+.45)} ${tip[1]-8*Math.sin(angle+.45)}`);svg.append(head);}};
    path([port('pico','right',.35),port('tph','left',.35)],false,true);path([port('tph','right',.35),port('cora','left',.35)],false,true);path([port('cora','left',.75),port('tph','right',.75)],false,true);
    path([port('sensor','top'),port('pico','bottom')],true,true);
    const origin=port('tph','bottom'),scope=port('scope','top'),analyzer=port('analyzer','top'),lane=(origin[1]+scope[1])/2;
    path([origin,[origin[0],lane],[scope[0],lane],scope]);path([[origin[0],lane],[analyzer[0],lane],analyzer]);
  }
  let pending=0;const observer=new ResizeObserver(()=>{cancelAnimationFrame(pending);pending=requestAnimationFrame(()=>{pending=0;route();});});observer.observe(root);for(const card of Object.values(cards))observer.observe(card);
  return {root,dispose:()=>{observer.disconnect();cancelAnimationFrame(pending);}};
}
