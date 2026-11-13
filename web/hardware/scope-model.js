export const scopeSources={calibration:{label:'Probe compensation reference',volts:5,period:.001},pulse:{label:'Pico pulse · nominal',volts:3.3,period:.001},response:{label:'FPGA return · expected',volts:3.3,period:2}};
export const scopeInput={ohms:1000000,farads:13e-12,source:'https://www.hantek.com/uploadpic/hantek/files/20250326/DSO2000%20Series%20Digital%20Storage%20Oscilloscope%20User%20Manual20250326154940.pdf'};

export function scopeNetlist({source='pulse',probe=10,compensation='matched'}={}){
  const signal=scopeSources[source];if(!signal||![1,10].includes(probe)||!['under','matched','over'].includes(compensation))throw new Error('Unsupported probe model');
  const capacitance=scopeInput.farads/9*({under:.65,matched:1,over:1.5}[compensation]),node=probe===10?'stimulus':'input',duration=signal.period*2.5,step=signal.period/2000;
  return ['Hantek input and ideal probe reference',`Vsignal ${node} 0 PULSE(0 ${signal.volts} 0 1u 1u ${signal.period/2-1e-6} ${signal.period})`,...(probe===10?[`Rprobe stimulus input 9meg`,`Cprobe stimulus input ${capacitance}`]:[]),'Rscope input 0 1meg','Cscope input 0 13p','.options reltol=1e-6 vntol=1e-9 abstol=1e-12 trtol=1',`.tran ${step} ${duration}` ,'.end'].join('\n');
}

export function readScopeResult(result,{source='pulse',channel=10}={}){
  if(![1,10].includes(channel)||!scopeSources[source])throw new Error('Unsupported scope settings');
  const time=result.data?.find(item=>item.name==='time')?.values,voltage=result.data?.find(item=>item.name==='v(input)')?.values;
  if(result.dataType!=='real'||!time?.length||time.length!==voltage?.length||!time.every(Number.isFinite)||!voltage.every(Number.isFinite))throw new Error('No valid transient voltage result');
  const values=voltage.map(value=>value*channel),max=Math.max(...values),min=Math.min(...values);
  return {time,voltage:values,vpp:max-min,min,max,engine:result.header.match(/ngspice-[\d.]+/)?.[0]||'ngspice',source:scopeSources[source].label,scope:'Ideal source and lumped probe/input equivalent; not a measured supplied probe, GPIO driver, cable or FPGA model.'};
}
