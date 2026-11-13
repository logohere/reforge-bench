const limit=500000;
export function waveformMetrics(time,voltage){
  if(!Array.isArray(time)||!Array.isArray(voltage)||time.length<3||time.length>limit||time.length!==voltage.length)throw new Error('Use 3–500,000 samples with matching time and voltage columns.');
  let min=Infinity,max=-Infinity;for(let i=0;i<time.length;i++){if(!Number.isFinite(time[i])||!Number.isFinite(voltage[i])||i&&time[i]<=time[i-1])throw new Error('Samples must be finite, with strictly increasing time.');min=Math.min(min,voltage[i]);max=Math.max(max,voltage[i]);}
  const threshold=(min+max)/2,rises=[];let highSeconds=0;
  for(let i=1;i<time.length;i++){const a=voltage[i-1],b=voltage[i],dt=time[i]-time[i-1];if(a<threshold&&b>=threshold)rises.push({time:time[i-1]+dt*(threshold-a)/(b-a),highSeconds});if(a>=threshold&&b>=threshold)highSeconds+=dt;else if((a>=threshold)!==(b>=threshold)){const fraction=(threshold-a)/(b-a);highSeconds+=dt*(a>=threshold?fraction:1-fraction);}}
  const span=time.at(-1)-time[0],completeCycles=Math.max(0,rises.length-1),cycleSpan=completeCycles?rises.at(-1).time-rises[0].time:0,frequency=max-min>1e-9&&completeCycles?completeCycles/cycleSpan:null;
  return {samples:time.length,span,min,max,vpp:max-min,risingEdges:rises.length,completeCycles,frequency,duty:frequency===null?null:(rises.at(-1).highSeconds-rises[0].highSeconds)/cycleSpan};
}

export function parseScopeCapture(text){
  if(typeof text!=='string'||text.length>64*1024*1024)throw new Error('Capture text must be at most 64 MiB.');
  let time,voltage,channel,declaredOrigin='unverified import',format;
  if(text.trimStart().startsWith('{')){
    const data=JSON.parse(text);
    if(data.schema!=='rea-scope-trace/1')throw new Error('Unsupported JSON capture schema.');
    ({time,voltage}=data);channel=data.channel||'CH1';declaredOrigin=data.origin==='simulation'?'simulation':'unverified import';format=data.schema;
  }else{
    const lines=text.replace(/^\uFEFF/,'').split(/\r?\n/),split=line=>line.split(/[\t,;]/).map(value=>value.trim().replace(/^"|"$/g,'')),clean=value=>value.toLowerCase().replace(/\s/g,'');let header=-1,t=-1,v=-1;
    for(let i=0;i<Math.min(lines.length,40);i++){const fields=split(lines[i]).map(clean),known=fields[0]==='nr'&&fields[1]==='time'&&fields.includes('ch1');t=fields.findIndex(value=>['time(s)','time[s]','time_s'].includes(value)||known&&value==='time');v=fields.findIndex(value=>['ch1(v)','ch1[v]','voltage(v)','voltage[v]','voltage_v'].includes(value)||known&&value==='ch1');if(t>=0&&v>=0){header=i;format=known?'phmarek SCPI CSV/TSV':'seconds/volts CSV';break;}}
    if(header<0)throw new Error('Use Time(s),Voltage(V) columns or the documented phmarek SCPI CSV export. Unknown units are not guessed.');
    time=[];voltage=[];channel='CH1';for(let i=header+1;i<lines.length;i++){if(!lines[i].trim())continue;const fields=split(lines[i]);if(!fields[t]||!fields[v])throw new Error('Missing sample on line '+(i+1));time.push(Number(fields[t]));voltage.push(Number(fields[v]));if(time.length>limit)throw new Error('Reduce the exported record to at most 500,000 samples.');}
    if(lines.slice(0,header).includes('# origin=simulation'))declaredOrigin='simulation';
  }
  const metrics=waveformMetrics(time,voltage),origin=declaredOrigin==='simulation'?'simulation':'import';
  return {time:time.map(value=>value-time[0]),voltage,channel,origin,declaredOrigin,format,...metrics,engine:origin==='simulation'?'simulation replay':'imported file',source:channel+' · '+(origin==='simulation'?'simulation replay':'imported waveform')};
}

export function comparePulseCapture(capture){
  const bounds={frequencyHz:[950,1050],highVolts:[2.97,3.63],lowVolts:[-.1,.1],duty:[.45,.55]},checks={frequency:capture.frequency===null?null:capture.frequency>=bounds.frequencyHz[0]&&capture.frequency<=bounds.frequencyHz[1],high:capture.max>=bounds.highVolts[0]&&capture.max<=bounds.highVolts[1],low:capture.min>=bounds.lowVolts[0]&&capture.min<=bounds.lowVolts[1],duty:capture.duty===null||capture.completeCycles<2?null:capture.duty>=bounds.duty[0]&&capture.duty<=bounds.duty[1]};
  return {status:Object.values(checks).includes(false)?'outside demo limits':Object.values(checks).includes(null)?'insufficient cycles':'within demo limits',checks,bounds,method:'Sample extrema; linear midpoint crossings; duty over complete rising-edge cycles (at least two). Demo limits, not IEEE 181 conformance or calibrated uncertainty.'};
}
