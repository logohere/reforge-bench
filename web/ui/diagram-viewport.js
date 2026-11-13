import {element,commandButton} from './components.js';

export function diagramViewport(svg){
  const root=element('div',undefined,'ui-diagram-viewport'),tools=element('div',undefined,'ui-diagram-navigation'),out=commandButton('−',{kind:'secondary',size:'sm'}),fit=commandButton('Fit',{kind:'secondary',size:'sm'}),zoom=element('output','100%'),inside=commandButton('+',{kind:'secondary',size:'sm'});
  out.setAttribute('aria-label','Zoom out');inside.setAttribute('aria-label','Zoom in');tools.setAttribute('role','group');tools.setAttribute('aria-label','Diagram navigation');zoom.setAttribute('aria-label','Diagram zoom');svg.setAttribute('tabindex','0');svg.setAttribute('aria-keyshortcuts','ArrowUp ArrowDown ArrowLeft ArrowRight + - f');
  tools.append(out,fit,zoom,inside);root.append(svg,tools);let base={x:0,y:0,width:950,height:650},view={...base},touched=false,drag=null,suppress=false,clickTimer=0;
  function paint(){svg.setAttribute('viewBox',[view.x,view.y,view.width,view.height].join(' '));const value=base.width/view.width;root.dataset.zoom=String(value);zoom.value=Math.round(value*100)+'%';out.disabled=value<=.5;inside.disabled=value>=16;}
  function fitTo(next,{reset=true}={}){base={...next};if(reset||!touched){view={...base};touched=false;}paint();}
  function reset(){view={...base};touched=false;paint();}
  function zoomBy(factor,anchor){const value=Math.min(16,Math.max(.5,base.width/view.width*factor)),ratio=view.width/(base.width/value),point=anchor||{x:view.x+view.width/2,y:view.y+view.height/2};view={x:point.x-(point.x-view.x)/ratio,y:point.y-(point.y-view.y)/ratio,width:view.width/ratio,height:view.height/ratio};touched=true;paint();}
  function point(event){return new DOMPoint(event.clientX,event.clientY).matrixTransform(svg.getScreenCTM().inverse());}
  out.addEventListener('click',()=>zoomBy(1/1.4));inside.addEventListener('click',()=>zoomBy(1.4));fit.addEventListener('click',reset);
  svg.addEventListener('wheel',event=>{event.preventDefault();zoomBy(Math.exp(-Math.max(-100,Math.min(100,event.deltaY))*.008),point(event));},{passive:false});
  svg.addEventListener('pointerdown',event=>{if(event.button!==0&&event.button!==1)return;const matrix=svg.getScreenCTM();drag={id:event.pointerId,x:event.clientX,y:event.clientY,view:{...view},scale:matrix.a,moved:false};});
  svg.addEventListener('pointermove',event=>{if(!drag||drag.id!==event.pointerId)return;const dx=event.clientX-drag.x,dy=event.clientY-drag.y;if(!drag.moved&&Math.hypot(dx,dy)<4)return;if(!drag.moved){drag.moved=true;svg.setPointerCapture(event.pointerId);}view={...drag.view,x:drag.view.x-dx/drag.scale,y:drag.view.y-dy/drag.scale};touched=true;root.dataset.panning='true';paint();});
  function release(event){if(!drag||drag.id!==event.pointerId)return;suppress=drag.moved;if(svg.hasPointerCapture(event.pointerId))svg.releasePointerCapture(event.pointerId);drag=null;root.dataset.panning='false';clearTimeout(clickTimer);clickTimer=setTimeout(()=>{suppress=false;},0);}
  svg.addEventListener('pointerup',release);svg.addEventListener('pointercancel',release);svg.addEventListener('click',event=>{if(suppress){event.preventDefault();event.stopImmediatePropagation();suppress=false;}},true);
  svg.addEventListener('keydown',event=>{if(event.ctrlKey||event.metaKey||event.altKey)return;const shifts={ArrowLeft:[-1,0],ArrowRight:[1,0],ArrowUp:[0,-1],ArrowDown:[0,1]};if(shifts[event.key]){event.preventDefault();const [x,y]=shifts[event.key],scale=svg.getScreenCTM().a;view.x+=x*60/scale;view.y+=y*60/scale;touched=true;paint();}else if(['+','=','-','f','F'].includes(event.key)){event.preventDefault();if(event.key.toLowerCase()==='f')reset();else zoomBy(event.key==='-'?1/1.4:1.4);}});
  paint();return {root,fitTo,reset,zoomBy,get view(){return {...view};},get state(){return {base:{...base},view:{...view}};},restore(state){base={...state.base};view={...state.view};touched=true;paint();},dispose(){clearTimeout(clickTimer);}};
}
