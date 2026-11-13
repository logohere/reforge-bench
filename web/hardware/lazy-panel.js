import {element as node,commandButton} from '../ui/components.js';

export function lazyPanel(panel,label,factory){
  let view=null,pending=null,disposed=false,active=true;panel.dataset.loadState='idle';
  function load(){
    if(disposed)return Promise.reject(new Error('View closed'));
    if(view)return Promise.resolve(view);
    if(pending)return pending;
    const status=node('p','Loading '+label+'…'),skeleton=node('div',undefined,'hardware-lazy-skeleton');status.setAttribute('role','status');skeleton.setAttribute('aria-hidden','true');for(let i=0;i<3;i++)skeleton.append(node('cds-skeleton-placeholder'));panel.replaceChildren(status,skeleton);panel.dataset.loadState='loading';panel.setAttribute('aria-busy','true');
    pending=Promise.resolve().then(()=>{if(disposed)throw new Error('View closed');return factory();}).then(result=>{
      if(disposed){result.dispose();throw new Error('View closed');}
      view=result;view.setActive?.(active);panel.replaceChildren(view.root);panel.dataset.loadState='ready';return view;
    }).catch(error=>{
      if(!disposed){const message=node('p',label+' could not be loaded.'),retry=commandButton('Retry',{kind:'secondary'});message.setAttribute('role','alert');retry.addEventListener('click',()=>void load().catch(()=>{}));panel.replaceChildren(message,retry);panel.dataset.loadState='error';}throw error;
    }).finally(()=>{pending=null;panel.setAttribute('aria-busy','false');});
    return pending;
  }
  return {load,setActive(value){active=value;view?.setActive?.(active);},dispose(){disposed=true;view?.dispose();view=null;}};
}
