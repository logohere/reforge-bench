// Device projects group evidence without changing exact-image case identities.
const SHA=/^[a-f0-9]{64}$/;
let projectsPromise;
const projects=new Map();
export function validateProjectIndex(index){
  if(index?.schema!=='rea-device-project-index/1'||!Array.isArray(index.projects))throw new Error('Device project index is invalid.');
  const assigned=new Set();
  for(const project of index.projects){
    if(!project.id||!project.name||!SHA.test(project.canonical_case_id)||!Array.isArray(project.case_ids)||!project.case_ids.includes(project.canonical_case_id)||!/^\/devices\/[a-z0-9-]+\.json$/.test(project.manifest))throw new Error('Device project identity is invalid.');
    for(const id of project.case_ids){if(!SHA.test(id)||assigned.has(id))throw new Error('Ambiguous device project membership.');assigned.add(id);}
  }
  return index;
}
export function decorateDeviceCases(cases,index){
  validateProjectIndex(index);
  const membership=new Map(index.projects.flatMap(project=>project.case_ids.map(id=>[id,project])));
  return cases.map(item=>membership.has(item.case_id)?{...item,device_project:membership.get(item.case_id)}:item);
}
export async function projectJson(url){
  if(!/^\/(devices|hardware)\/[a-zA-Z0-9_./-]+$/.test(url)||url.includes('..'))throw new Error('Invalid project resource.');
  const response=await fetch(url,{cache:'no-store',signal:AbortSignal.timeout(10000)});
  if(!response.ok)throw new Error(`Project evidence unavailable (${response.status}): ${url}`);
  return response.json();
}
export async function loadProjectIndex(){
  if(!projectsPromise)projectsPromise=projectJson('/devices/index.json').then(validateProjectIndex).catch(error=>{projectsPromise=null;throw error;});
  return projectsPromise;
}
export async function loadProjectLibrary(cases){
  const index=await loadProjectIndex();
  await Promise.all(index.projects.map(async entry=>{
    if(projects.has(entry.id))return;
    const project=await projectJson(entry.manifest);
    if(project.schema!=='rea-device-project/1'||project.id!==entry.id||project.canonical_case_id!==entry.canonical_case_id||JSON.stringify(project.case_ids)!==JSON.stringify(entry.case_ids))throw new Error('Device project manifest identity differs from its index.');
    projects.set(entry.id,project);
  }));
  return decorateDeviceCases(cases,index);
}
export function projectForCase(item){return projects.get(item?.device_project?.id)||null;}

export function filterProjectAnalyses(project,{query='',image='',profile=''}={}){
  const term=query.trim().toLowerCase();
  return (project.analyses||[]).filter(item=>(!image||item.image_sha256===image)&&(!profile||item.analysis_profile_sha256===profile)&&(!term||[item.name,item.address,item.image_sha256,item.analysis_profile_sha256,item.evidence_id].join(' ').toLowerCase().includes(term)));
}
export function validateDossier(entry,dossier){
  if(dossier?.subject?.digest?.sha256!==entry.image_sha256||dossier?.analysis_profile?.digest!==entry.analysis_profile_sha256||dossier?.evidence_id!==entry.evidence_id)throw new Error('Dossier image, analysis profile or evidence identity differs from the index.');
  const result=dossier.normalized_result;
  if(entry.address&&result?.procedure?.address!==entry.address)throw new Error('Dossier address differs from the index.');
  return result;
}
