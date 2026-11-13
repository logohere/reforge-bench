export const breadboardSpec={rows:63,columns:'ABCDEFGHIJ',pitch:2.54,centerGap:7.62,terminalContacts:630,railContacts:200,source:'https://lonelybinary.com/products/electronics-starter-kit-400-pieces'};
export function breadboardContact(id){const match=/^([A-J])([1-9]|[1-5][0-9]|6[0-3])$/.exec(id);if(!match)throw new Error('Unknown breadboard contact: '+id);const column=breadboardSpec.columns.indexOf(match[1]),row=Number(match[2]);return {id,row,column:match[1],strip:(column<5?'AE':'FJ')+row,position:[-95+(row-32)*2.54,13,80-(column<5?-(13.97-column*2.54):3.81+(column-5)*2.54)]};}
export function sameTerminalStrip(a,b){return breadboardContact(a).strip===breadboardContact(b).strip;}
export function coveredByPico(id){const point=breadboardContact(id).position;return point[0]>=-119.23&&point[0]<=-67.58&&Math.abs(point[2]-80)<=10.5;}
export const pulseLoopContacts={pulse:{source:'A26',feed:'B55',output:'A55',scope:'E55',analyzer:'D55'},ground:{source:'A25',feed:'B45',output:'A45',scope:'E45',scope2:'D45',analyzer:'C45'},response:{output:'A60',scope:'E60',analyzer:'D60'}};

export const terminalContacts=Array.from({length:63},(_,row)=>[...breadboardSpec.columns].map(column=>breadboardContact(column+(row+1)))).flat();
export const railContacts=Array.from({length:200},(_,index)=>{const side=index<100?-1:1,rail=Math.floor(index/50)%2,i=index%50,bank=side<0?'upper':'lower',polarity=(side<0?rail===0:rail===1)?'+':'−';return {id:bank+'-'+rail+'-'+(i+1),bank,rail,index:i+1,polarity,label:(side<0?'Top':'Bottom')+' '+polarity+' / '+(i+1),position:[-95+(Math.floor(i/5)*6+i%5-29)*2.54,13,80+side*(21+rail*2.54)]};});
export function breadboardConnection(id,{rails='unverified'}={}){
  if(!['unverified','continuous','split'].includes(rails))throw new Error('Unknown rail topology');
  if(/^[A-J]/.test(id)){const contact=breadboardContact(id),contacts=terminalContacts.filter(point=>point.strip===contact.strip);return {contact,contacts,verified:true,group:contact.strip};}
  const contact=railContacts.find(point=>point.id===id);if(!contact)throw new Error('Unknown breadboard contact: '+id);
  const contacts=railContacts.filter(point=>point.bank===contact.bank&&point.rail===contact.rail&&(rails==='continuous'||rails==='split'&&Math.floor((point.index-1)/25)===Math.floor((contact.index-1)/25)||point.id===id));
  return {contact,contacts,verified:rails!=='unverified',group:rails==='unverified'?id:contact.bank+'-'+contact.rail+(rails==='split'?'-'+Math.floor((contact.index-1)/25):'')};
}
export const picoPins=['GP0','GP1','GND','GP2','GP3','GP4','GP5','GND','GP6','GP7','GP8','GP9','GND','GP10','GP11','GP12','GP13','GND','GP14','GP15','GP16','GP17','GND','GP18','GP19','GP20','GP21','GND','GP22','RUN','GP26','GP27','AGND','GP28','ADC_VREF','3V3_OUT','3V3_EN','GND','VSYS','VBUS'].map((name,i)=>({number:i+1,name,contact:(i<20?'C'+(23+i):'H'+(62-i))}));
