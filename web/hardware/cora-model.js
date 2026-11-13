// Shared component envelopes traced from the numbered Digilent Cora board overview.
// Coordinates are illustrative millimetres, not manufacturing CAD or pad locations.
export const coraComponents=[
  ['J1',-8.8,-24.2,48.4,5.2,8,'socket'],['J5',0,23,41,5.2,8,'socket'],
  ['5V',-39.3,-30.4,9.4,16,11,'metal'],['J12',-22,-30,6.5,5.8,5,'metal'],
  ['FTDI',-23.3,-18.8,8.8,9,2,'chip'],['RJ45',-53,-8.2,22,16.2,16,'metal'],
  ['HOST',-43.8,14.9,13.7,15.9,12,'metal'],['ZYNQ',2.8,-7.7,16.7,17,2,'chip'],
  ['DDR3L',23.6,-6.6,9.8,14.6,2,'chip'],['JA',44.5,-18.7,9,15,7,'socket'],
  ['JB',44.5,4,9,15,7,'socket'],['BTN1',-24.4,22.2,5.5,5.5,3,'button'],
  ['BTN0',-12.4,22.2,5.5,5.5,3,'button'],['LD1',-22,17,2.6,2.6,1.5,'led'],
  ['LD0',-10,17,2.6,2.6,1.5,'led'],['JP3',-43.6,-28,3,8.5,4,'jumper'],
  ['MODE',42,24,5,2.5,3,'jumper'],['J7',35.5,-1,5,6.5,4,'jumper'],
  ['PWR',-48,-20.4,2,2,1.5,'led'],['DONE',-48,20,2,2,1.5,'led']
];
export function coraModel(item,{x=0,y=0}={}){
  const root=item('g',{transform:`translate(${x} ${y})`,class:'cora-model','data-board-model':'cora-z7-07s'});
  root.append(item('rect',{x:-50.8,y:-28.95,width:101.6,height:57.9,rx:.5,class:'cora-model-pcb'}));
  for(const px of [-47,47])for(const py of [-25,25])root.append(item('circle',{cx:px,cy:py,r:1.6,class:'cora-model-hole'}));
  for(const [name,px,py,w,d,height,type] of coraComponents){
    const group=item('g',{'data-board-feature':name});group.append(item('rect',{x:px,y:py,width:w,height:d,rx:.3,class:'cora-model-'+(type==='button'?'metal':type)}));
    if(type==='button')group.append(item('circle',{cx:px+w/2,cy:py+d/2,r:1.7,class:'cora-model-button'}));
    if(name==='J1'||name==='J5')for(let col=0;col<Math.floor(w/2.54);col++)for(const row of [0,1])group.append(item('rect',{x:px+1+col*2.54,y:py+.7+row*2.54,width:.8,height:.8,class:'cora-model-contact'}));
    if(name==='JA'||name==='JB')for(let row=0;row<6;row++)group.append(item('path',{d:`M${px-4} ${py+1.15+row*2.54}h4`,class:'cora-model-lead'}));
    const inside=['FTDI','RJ45','HOST','ZYNQ','DDR3L','5V','J12'].includes(name);
    if(inside)group.append(item('text',{x:px+w/2,y:py+d/2+.6,'text-anchor':'middle','font-size':name==='ZYNQ'?2.8:1.8,class:'cora-model-label '+(type==='metal'?'cora-model-label-on-metal':'')},name));
    else if(!['J1','J5','J7','PWR','DONE','JP3','MODE'].includes(name))group.append(item('text',{x:name==='JA'||name==='JB'?px-5:px+w/2,y:name==='JA'||name==='JB'?py+8:py-1.2,'text-anchor':name==='JA'||name==='JB'?'end':'middle','font-size':1.8,class:'cora-model-label'},name));
    root.append(group);
  }
  for(const [text,px,py,size] of [['CORA Z7',19,-14,3.6],['DIGILENT',13,18,2.6],['JP3',-44,-15.8,1.8],['MODE',44,22,1.6],['PWR',-44,-18.7,1.5],['DONE',-44,22,1.5]])root.append(item('text',{x:px,y:py,'text-anchor':'middle','font-size':size,class:'cora-model-label'},text));
  return root;
}

export function probeModel(item,probe){
  const angle=Math.atan2(probe.tail[0]-probe.tip[0],probe.tail[2]-probe.tip[2])*180/Math.PI;
  const root=item('g',{transform:`translate(${probe.tip[0]} ${probe.tip[2]}) rotate(${-angle})`,class:'probe-model','data-probe-model':probe.net});
  const part=(tag,attributes,feature)=>{const element=item(tag,{...attributes,'data-probe-feature':feature});root.append(element);return element;};
  part('path',{d:'M0 0L-.6 9H.6Z',class:'probe-model-metal'},'tip');
  part('path',{d:'M-1.8 7H1.8L2.5 25H-2.5Z',class:'probe-model-shell'},'hook-sleeve');
  part('path',{d:'M.7 0V-2.5H-1.7V1',class:'probe-model-hook'},'spring-hook');
  part('rect',{x:-3,y:17,width:6,height:4,rx:1,class:'probe-model-metal'},'ground-collar');
  part('path',{d:'M-4 25H4L4.8 34V71L3 78H-3L-4.8 71V34Z',class:'probe-model-shell'},'barrel');
  part('rect',{x:-7,y:28,width:14,height:3.5,rx:1.2,class:'probe-model-guard'},'finger-guard');
  part('rect',{x:-1.8,y:46,width:3.6,height:11,rx:.6,class:'probe-model-metal'},'attenuation-switch');
  part('rect',{x:-1.3,y:47,width:2.6,height:4,rx:.3,class:'probe-model-shell'},'switch-knob');
  for(let y=35;y<44;y+=2)root.append(item('path',{d:`M-4.5 ${y}H4.5`,class:'probe-model-grip'}));
  part('path',{d:'M-2.6 78H2.6L1.5 95H-1.5Z',class:'probe-model-shell'},'strain-relief');
  for(let y=80;y<93;y+=2.5)root.append(item('path',{d:`M-2 ${y}H2`,class:'probe-model-grip'}));
  root.append(item('text',{x:8,y:8,class:'hardware-cora-copy'},'Hook tip → E55'),item('text',{x:8,y:20,class:'hardware-cora-copy'},'Ground clip → E45'),item('text',{x:8,y:49,class:'hardware-cora-copy'},'CH1 · normal probe'),item('text',{x:8,y:57,class:'hardware-cora-note'},'10× switch'),item('text',{x:8,y:89,class:'hardware-cora-copy'},'Cable → CH1 BNC'));
  return root;
}
