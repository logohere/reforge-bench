let versionRequest;
export async function showBuild(label){
  try{const [response,native]=await Promise.all([fetch('/build.json',{cache:'no-store'}),fetch('/desktop-build.json',{cache:'no-store'})]);if(!response.ok)throw new Error('Build unavailable');const {id,scope}=await response.json();if(!/^[a-f0-9]{12}$/.test(id))throw new Error('Invalid build');const build=native.ok?await native.json():null;label.textContent=build?.schema==='rea-desktop-build/1'?'Build '+build.build_number+' · '+id:'UI '+id;label.title=scope;label.hidden=false;}catch{label.hidden=true;}
}
export async function showVersion(label){
  try{
    versionRequest??=fetch('/version.json',{cache:'no-store'}).then(async response=>{
      if(!response.ok)throw new Error('Version unavailable');
      const {version}=await response.json();
      if(!/^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version))throw new Error('Invalid app version');
      return version;
    });
    label.textContent='v'+await versionRequest;
    label.title='Firmware Bench application version, not manufacturer firmware version';
    label.hidden=false;
  }catch{label.hidden=true;}
}
