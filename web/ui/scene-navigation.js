import {Vector3} from 'three';

export function sceneNavigation(surface,camera,controls,{draw,onMove=()=>{},onReset=()=>{},onSpace=()=>{},canMove=()=>true}={}){
  const keys=new Set(),forward=new Vector3(),right=new Vector3(),up=new Vector3(0,1,0);let frame=0,last=0;
  surface.tabIndex=0;surface.setAttribute('aria-label','3D bench. Drag to orbit; W A S D to move; Q down; E up; Shift faster; Escape reset view; Space next guide step.');surface.setAttribute('aria-keyshortcuts','W A S D Q E Escape Space');
  function stop(){keys.clear();cancelAnimationFrame(frame);frame=0;last=0;}
  function tick(now){frame=0;if(!keys.size)return;const dt=last?Math.min((now-last)/1000,.05):1/60;last=now;camera.getWorldDirection(forward);forward.y=0;forward.normalize();right.crossVectors(forward,up).normalize();const delta=new Vector3();if(keys.has('w'))delta.add(forward);if(keys.has('s'))delta.sub(forward);if(keys.has('d'))delta.add(right);if(keys.has('a'))delta.sub(right);if(keys.has('e'))delta.add(up);if(keys.has('q'))delta.sub(up);delta.normalize().multiplyScalar(dt*(keys.has('shift')?600:200));
    const steps=Math.max(1,Math.ceil(delta.length()/4)),step=delta.divideScalar(steps);for(let i=0;i<steps;i++){const next=camera.position.clone().add(step);if(!canMove(next))break;camera.position.copy(next);controls.target.add(step);onMove();}controls.update();draw();frame=requestAnimationFrame(tick);
  }
  function down(event){const key=event.key.toLowerCase();if(event.ctrlKey||event.metaKey||event.altKey)return;if(key==='escape'||key===' '){event.preventDefault();stop();if(!event.repeat)(key==='escape'?onReset:onSpace)();return;}if(!['w','a','s','d','q','e','shift'].includes(key))return;event.preventDefault();keys.add(key);if(!frame)frame=requestAnimationFrame(tick);}
  function release(event){keys.delete(event.key.toLowerCase());if(![...keys].some(key=>'wasdqe'.includes(key)))stop();}
  function focus(){surface.focus({preventScroll:true});}
  surface.addEventListener('pointerdown',focus);surface.addEventListener('keydown',down);surface.addEventListener('keyup',release);surface.addEventListener('blur',stop);window.addEventListener('blur',stop);
  return {dispose(){stop();surface.removeEventListener('pointerdown',focus);surface.removeEventListener('keydown',down);surface.removeEventListener('keyup',release);surface.removeEventListener('blur',stop);window.removeEventListener('blur',stop);}};
}
