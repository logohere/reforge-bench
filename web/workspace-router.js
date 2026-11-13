import {activateWorkspace} from './workspace-lifecycle.js';
import {mountSimulationStop} from './simulation-stop.js';
import {systemCase} from './system-routes.js';
import {syncWorkspaceTabs} from './device-context.js';

const routes=new Set(['/home/','/library/','/lab/','/rea/','/timeline/','/hardware/','/guides/']);
export function workspaceTarget(href,current,selected){
  const url=new URL(href,current),from=new URL(current);
  if([...from.searchParams.keys()].some(key=>!['case','system','view','panel','moment','search','vendor','type','support','sort','import'].includes(key)&&!(key==='program'&&from.searchParams.get(key)==='oem-check')))return null;
  if(url.origin!==from.origin||!routes.has(url.pathname))return null;
  if(systemCase(url.search)==='')return null;
  if([...url.searchParams.keys()].some(key=>!['case','system'].includes(key)&&!(key==='program'&&url.searchParams.get(key)==='oem-check')))return null;
  if((systemCase(url.search)||selected)!==(systemCase(from.search)||selected))return null;
  return url;
}

if(typeof document!=='undefined'){
  const script=document.querySelector('script[data-workspace-entry]');
  if(script){
    await import(script.dataset.workspaceEntry);
    const views=new Map();let current=location.pathname,pending=false,queued=null;
    const selected=()=>{try{return sessionStorage.getItem('rea.active-device.v1');}catch{return null;}};
    const publish=()=>{if(window.__REA_RUNE_COMPAT__){const stop=document.getElementById('simulation-estop');window.ipc?.postMessage(JSON.stringify({kind:'workspace-state',path:location.pathname+location.search+location.hash,case:selected(),stop_available:location.pathname==='/lab/'&&!!stop&&!stop.disabled&&!stop.hidden}));}};
    const styles=()=>[...document.querySelectorAll('link[rel=stylesheet]')].filter(link=>!link.disabled);
    const capture=()=>({root:document.querySelector('main'),styles:styles(),title:document.title,workspace:document.body.dataset.workspace,program:document.documentElement.dataset.program,url:location.href,scroll:[scrollX,scrollY]});
    views.set(current,capture());document.body.dataset.router='ready';
    history.replaceState({...history.state,reaWorkspace:true},'',location.href);
    const notice=document.createElement('p');notice.className='visually-hidden';notice.setAttribute('role','status');document.body.append(notice);
    function shell(path){
      syncWorkspaceTabs();
      let stop=document.getElementById('simulation-estop');if(path==='/lab/'&&!stop){mountSimulationStop(document.querySelector('.shell-actions'));stop=document.getElementById('simulation-estop');}if(stop)stop.hidden=path!=='/lab/';
      document.querySelectorAll('.hardware-shell-title').forEach(title=>{title.hidden=path!=='/hardware/';});
    }
    async function navigate(url,{pop=false}={}){
      if(pending){queued={url,pop};return;}
      if(url.pathname===current){if(!pop)history.pushState({reaWorkspace:true},'',url);window.dispatchEvent(new HashChangeEvent('hashchange'));publish();return;}
      pending=true;const started=performance.now();notice.textContent='Loading '+url.pathname.split('/')[1]+'…';document.querySelector('.workspace-tabs')?.setAttribute('aria-busy','true');
      const previous=capture();views.set(current,previous);
      try{
        let view=views.get(url.pathname),entry;
        if(!view){
          const response=await fetch(url,{signal:AbortSignal.timeout(10000)});if(!response.ok)throw new Error('Workspace unavailable');
          const page=new DOMParser().parseFromString(await response.text(),'text/html');
          entry=page.querySelector('script[data-workspace-entry]')?.dataset.workspaceEntry;if(!entry)throw new Error('Workspace entry unavailable');
          const root=page.querySelector('main');if(!root)throw new Error('Workspace content unavailable');
          for(const element of root.querySelectorAll('[src],[href]'))for(const attribute of ['src','href']){const value=element.getAttribute(attribute);if(value&&!value.startsWith('#'))element.setAttribute(attribute,new URL(value,url).href);}
          const links=[];
          await Promise.all([...page.querySelectorAll('link[rel=stylesheet]')].map(async source=>{
            const href=new URL(source.getAttribute('href'),url).href;let link=[...document.querySelectorAll('link[rel=stylesheet]')].find(link=>link.href===href);
            if(!link){link=document.createElement('link');link.rel='stylesheet';link.href=href;const loaded=new Promise((resolve,reject)=>{link.onload=resolve;link.onerror=()=>reject(new Error('Workspace styles unavailable'));});document.head.append(link);await loaded;}links.push(link);
          }));
          view={root,styles:links,title:page.title,workspace:page.body.dataset.workspace,program:page.documentElement.dataset.program,url:String(url),scroll:[0,0]};
        }
        activateWorkspace(current,false);
        for(const link of document.querySelectorAll('link[rel=stylesheet]'))link.disabled=!view.styles.includes(link);
        previous.root.replaceWith(view.root);document.body.dataset.workspace=view.workspace;
        if(view.program)document.documentElement.dataset.program=view.program;else delete document.documentElement.dataset.program;
        delete document.documentElement.dataset.loading;document.querySelector('.workspace-loading')?.remove();
        if(!pop)history.pushState({reaWorkspace:true},'',entry?url:(url.hash?url:view.url));
        current=url.pathname;document.title=view.title;shell(current);
        if(entry){await import(entry);view={...view,...capture()};views.set(current,view);}
        activateWorkspace(current,true);shell(current);window.dispatchEvent(new HashChangeEvent('hashchange'));
        publish();
        scrollTo(...view.scroll);notice.textContent='';
        view.root.setAttribute('tabindex','-1');view.root.focus({preventScroll:true});
        performance.mark('rea:workspace-ready');
        performance.measure('rea:workspace',{start:started,end:performance.now(),detail:{path:current,cached:!entry}});
      }catch(error){notice.textContent=error.message+'. Open the page directly to retry.';location.assign(url);}
      finally{pending=false;document.querySelector('.workspace-tabs')?.removeAttribute('aria-busy');if(queued){const next=queued;queued=null;void navigate(next.url,{pop:next.pop});}}
    }
    document.addEventListener('click',event=>{
      if(event.defaultPrevented||event.button!==0||event.metaKey||event.ctrlKey||event.shiftKey||event.altKey)return;
      const link=event.composedPath().find(node=>node?.tagName==='A'||node?.tagName==='CDS-SIDE-NAV-LINK'||node?.tagName==='CDS-BUTTON'&&node.href);
      if(!link?.href||link.download||link.target&&link.target!=='_self')return;
      const url=workspaceTarget(link.href,location.href,selected());if(!url||url.pathname===current)return;
      event.preventDefault();void navigate(url);
    });
    window.addEventListener('popstate',()=>{const url=workspaceTarget(location.href,location.href,selected());if(url)void navigate(url,{pop:true});else location.reload();});
    if(window.__REA_RUNE_COMPAT__){
      window.addEventListener('rea-device-selected',publish);
      window.addEventListener('rea-simulation-stop-state',publish);
      window.addEventListener('rea-workspace-visible',event=>activateWorkspace(current,event.detail===true));
      const open=path=>{window.__reaPendingWorkspace=null;const target=new URL(path,location.href);if(target.origin!==location.origin)return;const url=workspaceTarget(target,location.href,selected());if(url)void navigate(url);else location.assign(target);};
      window.addEventListener('rea-workspace-open',event=>open(event.detail));
      if(window.__reaPendingWorkspace)open(window.__reaPendingWorkspace);
      publish();
    }
  }
}
