export function scopeAutoScale(result,span=result.time.at(-1)-result.time[0]){
  const {min,max}=result;
  if(!Number.isFinite(span)||span<=0||!Number.isFinite(min)||!Number.isFinite(max)||max<min||!Number.isFinite(max-min))throw new Error('Invalid waveform range');
  return {timebase:span/10,volts:(max-min)/6||1,offsetVolts:min-(max===min?3:0)};
}

export function scopeOffScale({result,volts,offsetVolts}){
  return !!result&&(result.min<offsetVolts-volts||result.max>offsetVolts+7*volts);
}

export function scopeDivision(value,unit){
  const scales=unit==='s'?[[1,'s'],[1e-3,'ms'],[1e-6,'µs'],[1e-9,'ns']]:[[1,'V'],[1e-3,'mV'],[1e-6,'µV']];
  const [scale,label]=scales.find(([scale])=>Math.abs(value)>=scale)||scales.at(-1);
  return {value:Number((value/scale).toPrecision(4)).toString(),unit:label};
}

export function drawScopePlot(ctx,state,{x,y,width,height}){
  ctx.lineWidth=1;ctx.strokeStyle='#26343d';ctx.beginPath();
  for(let column=0;column<=10;column++){const px=x+column*width/10;ctx.moveTo(px,y);ctx.lineTo(px,y+height);}
  for(let row=0;row<=8;row++){const py=y+row*height/8;ctx.moveTo(x,py);ctx.lineTo(x+width,py);}ctx.stroke();
  const {result,timebase,volts,offsetVolts}=state;
  if(!result||!Number.isFinite(timebase)||timebase<=0||!Number.isFinite(volts)||volts<=0)return;
  ctx.save();ctx.beginPath();ctx.rect(x,y,width,height);ctx.clip();ctx.strokeStyle='#e6cb54';ctx.lineWidth=2;ctx.beginPath();
  const origin=result.time[0];
  for(let index=0;index<result.time.length;index++){
    const px=x+(result.time[index]-origin)/(timebase*10)*width,py=y+height*7/8-(result.voltage[index]-offsetVolts)/volts*height/8;
    if(index===0)ctx.moveTo(px,py);else ctx.lineTo(px,py);
    if(px>x+width)break;
  }
  ctx.stroke();ctx.restore();
}
