import {element,actionLink} from '../ui/components.js';

export const wireConvention=[
  {id:'ground',name:'GND · black',description:'Common 0 V reference'},
  {id:'pulse',name:'Pulse · yellow',description:'Pico GP2 → Cora P1'},
  {id:'response',name:'Return · blue',description:'Cora P2 → analyzer 1'}
];
export function wiringLegend(profile='pulse'){
  const root=element('div',undefined,'hardware-wire-legend');root.setAttribute('aria-label','Physical jumper color convention');
  const convention=profile==='ddcs'?[{id:'ground',name:'GND · black',description:'Existing common ground'},{id:'pulse',name:'STEP · yellow',description:'FPGA P1 output · GP2 input'},{id:'response',name:'DIR · blue',description:'FPGA P2 output → analyzer CH1'},{id:'uartTx',name:'UART TX · green',description:'Pico GP0 → FPGA P3 RX'},{id:'uartRx',name:'UART RX · white',description:'FPGA P4 TX → Pico GP1'}]:wireConvention;
  for(const item of convention){const chip=element('span',item.name);chip.dataset.net=item.id;chip.title=item.description;root.append(chip);}
  const more=element('details'),summary=element('summary','Color convention and references');more.append(summary,element('p','Black is common ground; red is reserved for positive supply and stays unused in this USB-powered harness. Yellow pulse and blue return are this bench’s signal identifiers. Match names and pin numbers as well as color.'),element('p','These are low-voltage lab conventions. IEEE 315 concerns diagram symbols; IEC 60445 concerns conductor identification. This guide does not claim standards certification or apply GPIO colors to mains wiring.'),actionLink('MIT · wiring practice','https://web.mit.edu/6.101/www/s2019/handouts/L02.pdf'),actionLink('IEEE · diagram symbols','https://standards.ieee.org/ieee/315_/6396/'),actionLink('IEC · terminal and conductor identification','https://webstore.iec.ch/en/publication/66712'));root.append(more);return root;
}
