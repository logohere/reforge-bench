const roles={
  'ddcs-expert-2020':['DDCS Expert','CNC controller application','2020-09-28-00','ARM Linux executable · ELF'],
  'ddcs-v41-motiondev':['DDCS V4.1 motion driver','Linux driver research','Build dated 4 Apr 2025','Linux kernel module · ELF'],
  'ddcs-expert-m350-motiondev':['Expert / M350 motion driver','Related-controller driver research','Backup dated 31 Dec 2025','Linux kernel module · ELF'],
  'centroid-acorn-v482':['Acorn','CNC controller firmware','CNC12 v4.82','TI GP ARM image'],
  'dobot-magician-lite-2022':['Magician Lite','Robot firmware update','Archive dated 6 Jul 2022','DfuSe update container'],
  'roland-mdx40a-v120':['MDX-40A','Desktop mill firmware update','v1.20','Roland update container'],
};

export function systemInfo(item){
  const meta=item.library||{};
  const project=item.device_project;
  if(project)return {name:project.model||project.name,shortName:project.name,vendor:project.vendor,purpose:'Instrument firmware, hardware and measured experiments',family:project.model||project.name,version:'Collected device project',format:meta.artifact_format||'Retained component',component:meta.artifact_role||'Retained device component',group:'device-project:'+project.id};
  const fanuc=item.profile_id?.startsWith('fanuc-crx-v940p90-');
  const haas=item.profile_id?.startsWith('haas-coldfire2-');
  const known=roles[item.profile_id];
  return {name:fanuc?'CRX controller':known?.[0]||meta.model||'Unidentified firmware',
    shortName:({'haas-coldfire2-lathe':'Haas Classic Control · Lathe','haas-coldfire2-mill':'Haas Classic Control · Mill','haas-coldfire2-loader':'Haas ColdFire 2 · Loader'})[item.profile_id]||null,
    vendor:meta.vendor||'Unknown manufacturer',purpose:fanuc?'Robot controller update':haas?({'haas-coldfire2-lathe':'Lathe controller firmware','haas-coldfire2-mill':'Mill controller firmware','haas-coldfire2-loader':'Controller startup loader'})[item.profile_id]||meta.artifact_role:known?.[1]||meta.artifact_role||'Firmware investigation',
    family:fanuc?'CRX':haas?'Classic Control · ColdFire 2':item.profile_id?.startsWith('ddcs-')||/^DDCS\b/i.test(meta.model||'')?'DDCS':meta.family_name||meta.model||'Other firmware',
    version:fanuc?'V9.40P/90':known?.[2]||meta.version||'Version not identified',
    format:fanuc?'Packed FANUC load module':haas?'ColdFire firmware · mapped analysis ELF':known?.[3]||meta.artifact_format||'Format not identified',
    component:fanuc?'Internal package component · role not yet verified':known?.[1]||meta.artifact_role||'Role not yet established',
    group:fanuc?'fanuc-crx-v940p90':haas?'haas-coldfire2':meta.family_id?JSON.stringify([meta.vendor||null,meta.family_id,meta.version||null]):item.profile_id||item.case_id};
}

export function manufacturerGroups(groups){
  const brands=new Map();
  for(const group of groups){
    const info=systemInfo(group.items[0]),key=info.vendor.toLocaleLowerCase();
    if(!brands.has(key))brands.set(key,{vendor:info.vendor,families:new Map(),items:[]});
    const brand=brands.get(key);brand.items.push(...group.items);
    if(!brand.families.has(info.family))brand.families.set(info.family,{name:info.family,groups:[]});
    brand.families.get(info.family).groups.push(group);
  }
  return [...brands.values()].map(brand=>({...brand,families:[...brand.families.values()]}));
}

export function systemGroups(cases,selected){
  const groups=new Map();
  for(const item of cases){
    const info=systemInfo(item);
    if(!groups.has(info.group))groups.set(info.group,{...info,items:[]});
    groups.get(info.group).items.push(item);
  }
  return [...groups.values()].map(group=>({...group,item:group.items.find(item=>item.case_id===selected)||group.items.find(item=>item.device_project?.canonical_case_id===item.case_id)||group.items[0]}));
}
