import {tphGeometry} from './layout.js';

export function tphLabel(number){return number===5||number===11?'GND':number===6||number===12?'VCC':'P'+number;}
export function tphModel(item,{x=0,y=0}={}){
  const root=item('g',{transform:`translate(${x} ${y})`,class:'tph-model','data-board-model':'pmod-tph2'});
  root.append(item('rect',{x:-16.5,y:-10,width:33,height:20,rx:.4,class:'tph-model-pcb'}),item('rect',{x:-16,y:-8.2,width:2.5,height:16.4,class:'tph-model-header'}),item('rect',{x:13.5,y:-8.2,width:7,height:16.4,class:'tph-model-header'}));
  for(let row=0;row<6;row++){const py=(row-2.5)*2.54;root.append(item('rect',{x:-23,y:py-.32,width:9.5,height:.64,class:'tph-model-gold'}),item('path',{d:`M-13.5 ${py}H-7M7 ${py}H13.5`,class:'tph-model-trace'}),item('rect',{x:17,y:py-.5,width:1,height:1,class:'tph-model-socket'}));}
  root.append(item('rect',{x:-2.54,y:-7.62,width:5.08,height:15.24,class:'tph-model-header'}));
  for(const pin of tphGeometry.pins){const px=pin.position[0],py=pin.position[2],label=tphLabel(pin.number),tone=pin.number===1?'pulse':pin.number===2?'response':label==='GND'?'ground':label==='VCC'?'power':'unused';
    const cell=item('g',{'data-tph-pin':pin.number,'data-tone':tone});cell.append(item('circle',{cx:px,cy:py,r:.76,class:'tph-model-pad'}),item('rect',{x:px-.32,y:py-.32,width:.64,height:.64,class:'tph-model-gold'}),item('text',{x:px<0?-3.5:3.5,y:py+.6,'text-anchor':px<0?'end':'start',class:'tph-model-label'},label),item('title',{},label+' · connector pin '+pin.number));root.append(cell);
  }
  root.append(item('path',{d:'M-4.2 -8.8h2l-1 1.2Z',class:'tph-model-pin-one'}),item('text',{x:0,y:9.1,'text-anchor':'middle',class:'tph-model-brand'},'Pmod TPH2'));
  return root;
}
