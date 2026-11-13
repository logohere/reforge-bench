import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';

export function pulsePlot(root){
  root.setAttribute('role','img');root.setAttribute('aria-label','Simulated GP2 to JA1 input, JA2 return, RTL fault and BTN0 reset. Shared elapsed-seconds timebase. Read values with the replay slider and event buttons.');
  const marker=document.createElement('div');marker.className='hardware-experiment-cursor';marker.setAttribute('aria-hidden','true');
  let chart=null,scenario=null,start=0,end=2.60001,time=0;
  function position(current=chart){if(!current)return;const ratio=uPlot.pxRatio;marker.style.left=(current.bbox.left/ratio+current.valToPos(time,'x'))+'px';marker.style.top=current.bbox.top/ratio+'px';marker.style.height=current.bbox.height/ratio+'px';marker.hidden=time<start||time>end;}
  function resize(){const {width,height}=root.getBoundingClientRect();if(width<10||height<10||!scenario)return;if(chart){chart.setSize({width:Math.floor(width),height:Math.floor(height)});position();return;}const styles=getComputedStyle(root),color=token=>styles.getPropertyValue('--rb-color-'+token).trim(),font='16px '+styles.getPropertyValue('--rb-font-ui').trim(),data=[[],[],[],[],[]];for(const row of scenario.samples){data[0].push(row[0]/1e9);for(let i=1;i<=4;i++)data[i].push((4-i)*2+row[i]*.65);}
    chart=new uPlot({width:Math.floor(width),height:Math.floor(height),hooks:{draw:[current=>position(current)]},legend:{show:false},cursor:{show:false},select:{show:false},padding:[20,16,8,0],scales:{x:{time:false,auto:false,range:()=>[start,end]},y:{auto:false,range:()=>[-.4,7.2]}},axes:[{font,stroke:color('secondary'),size:42,space:100,values:(self,ticks)=>ticks.map(value=>value.toFixed(end-start<.02?3:2)+' s'),grid:{stroke:color('divider')},ticks:{show:false}},{font,stroke:color('text'),size:130,splits:()=>[0,2,4,6],values:()=>['BTN0 reset','RTL fault','JA2 return','GP2 → JA1'],grid:{stroke:color('divider')},ticks:{show:false}}],series:[{},...['Stimulus','Response','RTL fault','Reset'].map((label,i)=>({label,stroke:color(['info','success','danger','warning'][i]),width:1.5,paths:uPlot.paths.stepped({align:1}),points:{show:false}}))]},data,root);root.append(marker);position();
  }
  const observer=new ResizeObserver(resize);observer.observe(root);
  return {setScenario(next){scenario=next;chart?.destroy();chart=null;resize();},setWindow(from,to){if(start===from&&end===to)return;start=from;end=to;chart?.setScale('x',{min:start,max:end});position();},setPosition(value){time=value;position();},dispose(){observer.disconnect();chart?.destroy();marker.remove();}};
}
