export function recoveryProfile(item,profiles){
  const record=profiles[item.profile_id];
  return record?.sha256===item.case_id?record:null;
}

export function analysisProgress(item,profiles){
  const record=recoveryProfile(item,profiles);
  if(record?.compared===true)return {rank:300+(Number(record.rank)||0),label:'Bounded comparison',tone:'pass'};
  if(item.last_runs?.analyze?.ok===true)return {rank:200,label:'Inspected',tone:'info'};
  if(record?.milestones?.some(([name,state])=>name==='Analyze'&&['Partial','Analyzed'].includes(state)))return {rank:200,label:'Partial analysis',tone:'info'};
  if(item.last_runs?.analyze?.ok===false)return {rank:100,label:'Analysis failed',tone:'danger'};
  return {rank:0,label:'Acquired',tone:'muted'};
}
