import {analyzerGeometry,analyzerBreakout} from './layout.js';

export function analyzerModel(item,{x=0,y=0}={}){
  const root=item('g',{transform:`translate(${x} ${y})`,'data-instrument-model':'lonely-binary-analyzer'});
  const rect=(x,y,w,h,kind,extra={})=>root.append(item('rect',{x,y,width:w,height:h,rx:.7,class:'instrument-'+kind,...extra}));
  const text=(value,x,y,size=2,anchor='middle')=>root.append(item('text',{x,y,'text-anchor':anchor,'font-size':size,class:'instrument-text'},value));
  rect(-27.5,-13,55,26,'case');rect(-29,-4,3,8,'metal');rect(-29.2,-2.6,1,5.2,'dark');
  rect(-23.5,-10.5,47,21,'sticker');rect(-23.5,5,47,5.5,'brand');
  text('LOGIC',-21,-3,3,'start');text('ANALYZER',-21,1.2,3,'start');
  root.append(item('circle',{cx:-3,cy:-5.3,r:3,class:'instrument-logo'}),item('path',{d:'M-6 -5.3h1.1l.8-1.8 1.4 3.5 1.2-2.3H0',class:'instrument-logo'}));
  for(const [row,pair] of [['CH0','CH1'],['CH2','CH3'],['CH4','CH5'],['CH6','CH7'],['CLK','GND']].entries()){const py=-7.8+row*2.25;text(pair[0],9,py,1.65,'end');text(pair[1],16.5,py,1.65,'start');for(const px of [11,14])rect(px,py-1.3,1.4,1.4,'socket');}
  text('8CH',-18,9,2.5);text('24MHz',-7,9,2.5);text('lonely binary',12,8.9,2.1);
  rect(27.5,-6.7,5,13.4,'dark');for(const pin of analyzerGeometry.pins)rect(31.5,pin.position[2]-.25,5,.5,'gold');
  root.append(item('path',{d:'M33 -11H45V-8H66V8H45V11H33Z',class:'instrument-adapter','data-analyzer-adapter':'breadboard'}));
  for(const side of [-1,1])rect(38.2,side*5.08-1.27,25.4,2.54,'header');
  for(const contact of analyzerBreakout.contacts){const px=contact.position[0],py=contact.position[2];rect(px-.32,py-.32,.64,.64,'gold',{'data-breakout-signal':contact.name});}
  for(const [i,name] of analyzerBreakout.signals.entries()){const px=39.5+i*2.54;root.append(item('text',{x:px,y:0,'text-anchor':'middle','dominant-baseline':'middle',transform:`rotate(-90 ${px} 0)`,'font-size':1.5,class:'instrument-text'},name.replace('CH','')));}
  return root;
}

export function scopePanelModel(item,{x=0,y=0,rear=false}={}){
  const root=item('g',{transform:`translate(${x} ${y})`,'data-instrument-model':rear?'hantek-rear':'hantek-front'});
  const rect=(x,y,w,h,kind,r=1)=>root.append(item('rect',{x,y,width:w,height:h,rx:r,class:'instrument-'+kind}));
  const text=(value,x,y,size=3,anchor='middle')=>root.append(item('text',{x,y,'text-anchor':anchor,'font-size':size,class:'instrument-text'},value));
  rect(-159,-75,318,150,'case',5);
  if(rear){rect(-140,-52,190,85,'dark',2);for(let row=0;row<7;row++)for(let col=0;col<16;col++)rect(-134+col*11,-46+row*11,6,2,'vent',.5);rect(55,-25,27,29,'metal');rect(61,-19,15,17,'dark');text('USB-B · TO WINDOWS',69,19,4);rect(111,-40,32,45,'dark');text('AC MAINS',127,19,3.5);text('REAR · USB is data only; scope uses grounded mains',0,57,4);return root;}
  rect(-149,-53,164,103,'dark',2);rect(-144,-48,150,84,'screen',1);
  for(let i=0;i<=10;i++)root.append(item('path',{d:`M${-144+i*15} -48V36`,class:'instrument-grid'}));for(let i=0;i<=8;i++)root.append(item('path',{d:`M-144 ${-48+i*10.5}H6`,class:'instrument-grid'}));
  text('Hantek',-144,-62,7,'start');text('DSO2D15',-103,-62,5,'start');text('NO LIVE CAPTURE',-69,-8,5);text('CH1 normal probe · 10×',-69,3,4);
  for(let i=0;i<7;i++){root.append(item('ellipse',{cx:23,cy:-46+i*14,rx:5,ry:3.3,class:'instrument-key'}));text('F'+i,23,-44.9+i*14,2.5);}
  const key=(label,x,y,w=13)=>{rect(x-w/2,y-3.5,w,7,'key');text(label,x,y+1,1.9);};
  for(const [label,px,py] of [['SAVE',72,-64],['MEASURE',89,-64],['ACQUIRE',106,-64],['AUTO',124,-64],['RUN/STOP',142,-57],['UTILITY',73,-51],['CURSOR',90,-51],['DISPLAY',107,-51],['SINGLE',124,-51],['CH1',50,0],['MATH',67,0],['CH2',84,0],['HORIZ',115,0],['TRIG',142,-18],['FORCE',142,-5],['WAVE GEN',142,9]])key(label,px,py);
  const knob=(name,px,py,r=6)=>{root.append(item('circle',{cx:px,cy:py,r,class:'instrument-knob'}),item('path',{d:`M${px} ${py-r+1.5}v2.5`,class:'instrument-mark'}));text(name,px,py+r+4,2.1);};
  knob('V0 / MENU',50,-58,5);for(const px of [50,84])knob('POSITION',px,-18,5);knob('POSITION',115,-18,5);knob('LEVEL',142,-39,5);knob('VOLTS/DIV',50,22,7);knob('VOLTS/DIV',84,22,7);knob('SEC/DIV',115,22,7);
  for(const [name,px] of [['CH1',45],['CH2',76],['EXT / GEN',108]]){root.append(item('circle',{cx:px,cy:62,r:6,class:'instrument-metal','data-scope-port':name}),item('circle',{cx:px,cy:62,r:3.2,class:name==='CH1'?'instrument-ch1':'instrument-dark'}),item('circle',{cx:px,cy:62,r:1.2,class:'instrument-gold'}));text(name,px,51,3);}
  rect(-141,56,15,11,'key',2);text('POWER',-133.5,73,2.5);rect(-80,58,14,7,'metal');rect(-77,60,8,3,'dark');text('USB-A · STORAGE',-73,73,2.5);
  root.append(item('path',{d:'M138 56v12m-2-12h4m-4 12h4',class:'instrument-mark'}));text('PROBE COMP',137,46,2.3);
  return root;
}
