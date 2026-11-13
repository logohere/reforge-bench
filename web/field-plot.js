import uPlot from 'uplot';
import 'uplot/dist/uPlot.min.css';

export function fieldPlot(root,study){
  root.setAttribute('role','img');root.setAttribute('aria-label','Calculated Bx, By and Bz versus magnet rotation. Degrees and millitesla; point fields, not AS5600 angle output. Read exact values with the rotation control.');
  const marker=document.createElement('div');marker.className='hardware-field-cursor';marker.setAttribute('aria-hidden','true');
  const maximum=Math.ceil(Math.max(...study.scenarios.flatMap(item=>item.field_T.flatMap(row=>row.map(Math.abs))))*1000/10)*10;
  let chart=null,scenario=study.scenarios[0],angle=0;
  function position(current=chart){if(!current)return;const ratio=uPlot.pxRatio;marker.style.left=(current.bbox.left/ratio+current.valToPos(angle,'x'))+'px';marker.style.top=current.bbox.top/ratio+'px';marker.style.height=current.bbox.height/ratio+'px';}
  const data=()=>[study.angles_deg,...[0,1,2].map(axis=>scenario.field_T.map(row=>row[axis]*1000))];
  function resize(){
    if(!root.isConnected){observer.disconnect();chart?.destroy();chart=null;return;}const {width,height}=root.getBoundingClientRect();if(width<10||height<10)return;
    if(chart){chart.setSize({width:Math.floor(width),height:Math.floor(height)});position();return;}
    const styles=getComputedStyle(root),color=token=>styles.getPropertyValue('--rb-color-'+token).trim(),font='16px '+styles.getPropertyValue('--rb-font-ui').trim();
    chart=new uPlot({width:Math.floor(width),height:Math.floor(height),hooks:{draw:[current=>position(current)]},legend:{show:false},cursor:{show:false},select:{show:false},padding:[20,16,8,0],scales:{x:{time:false,auto:false,range:()=>[0,360]},y:{auto:false,range:()=>[-maximum,maximum]}},axes:[{font,stroke:color('secondary'),size:44,splits:()=>[0,90,180,270,360],values:(self,ticks)=>ticks.map(value=>value+'°'),grid:{stroke:color('divider')},ticks:{show:false}},{font,stroke:color('secondary'),size:72,label:'Field (mT)',labelFont:font,grid:{stroke:color('divider')},ticks:{show:false}}],series:[{},...['Bx','By','Bz'].map((label,i)=>({label,stroke:color(['info','success','warning'][i]),width:2,points:{show:false}}))]},data(),root);root.append(marker);position();
  }
  const observer=new ResizeObserver(resize);observer.observe(root);
  return {setScenario(value){scenario=value;chart?.setData(data());resize();},setAngle(value){angle=value;position();},dispose(){observer.disconnect();chart?.destroy();marker.remove();}};
}
