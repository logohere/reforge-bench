// Reuse the original shared bench scope, including geometry,
// simulated controls, navigation and graphics fallback. No physical USB/VISA
// connection is opened by this wrapper or its default bench component.
import {element as node} from '../ui/components.js';

export function mountHantekScope3D({
  loadBench=()=>import('/dist/hardware-scene.js'),
  createRoot=()=>node('div',undefined,'hantek-hardware-surface hantek-scope-existing'),
  createStatus=()=>node('p','Loading the original hardware bench scope model…','hantek-hardware-model-note')
}={}){
  const root=createRoot(),status=createStatus();root.append(status);
  let bench=null,disposed=false,active=true;
  const ready=(async()=>{
    const {mountHardwareBench}=await loadBench();
    if(disposed)return;
    bench=mountHardwareBench({mode:'3d',camera:'wiring',guide:false,isolated:true});
    root.replaceChildren(bench.root);
    await bench.ready;
    if(disposed)return;
    bench.inspectPart('scope');
    if(bench.root.dataset.graphics!=='unavailable')bench.setMode('3d');
    root.hidden=!active;
  })();
  return {root,ready,
    setActive(value){active=value;root.hidden=!value;},
    dispose(){if(disposed)return;disposed=true;bench?.dispose();}
  };
}
