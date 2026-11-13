import {element as node,commandButton,badge,propertyList,viewSwitcher} from '../ui/components.js';
import {fieldPlot} from '/dist/field-plot.js';
import {checkedMagneticStudy} from './magnetic-study.js';

export function magneticFieldStudy({study}){
  checkedMagneticStudy(study);
  const root=node('div',undefined,'hardware-field-study'),heading=node('div',undefined,'hardware-field-heading'),title=node('div'),cases=node('div',undefined,'hardware-field-cases'),plot=node('div',undefined,'hardware-field-plot'),readings=node('dl',undefined,'hardware-field-readings'),main=node('section',undefined,'hardware-field-main'),inspector=node('aside',undefined,'hardware-field-inspector');
  title.append(node('h2','Diametric magnet'),badge('Calculated study','info'));heading.append(title,node('p','Hypothetical magnet · not measured equipment'));cases.setAttribute('role','group');cases.setAttribute('aria-label','Magnetic field study case');
  let selected=study.scenarios[0],index=0;const buttons=new Map(),outputs=[];
  for(const scenario of study.scenarios){const button=commandButton(scenario.label,{kind:'ghost',size:'sm'});button.dataset.scenario=scenario.id;button.addEventListener('click',()=>{selected=scenario;chart.setScenario(selected);update();});buttons.set(scenario.id,button);cases.append(button);}
  for(const [i,label] of ['Bx','By','Bz','Magnitude'].entries()){const group=node('div'),value=node('dd');value.dataset.field=['bx','by','bz','magnitude'][i];group.append(node('dt',label+' · mT'),value);readings.append(group);outputs.push(value);}
  const position=node('input'),positionLabel=node('output'),positionRow=node('div',undefined,'hardware-field-position');position.type='range';position.min='0';position.max=String(study.angles_deg.length-1);position.step='1';position.value='0';position.setAttribute('aria-label','Calculated magnet rotation');positionRow.append(node('span','Rotation'),position,positionLabel);position.addEventListener('input',()=>{index=Number(position.value);update();});
  main.append(cases,readings,plot,positionRow);root.append(heading,main,inspector);const chart=fieldPlot(plot,study),geometry=node('div',undefined,'hardware-field-geometry'),method=node('div',undefined,'hardware-field-method'),parameters=node('div');
  const namespace='http://www.w3.org/2000/svg',svg=document.createElementNS(namespace,'svg');svg.setAttribute('viewBox','0 0 400 280');svg.setAttribute('role','img');svg.setAttribute('aria-label','Dimensioned XZ cross-section of the hypothetical cylinder and observation point');
  function part(tag,attributes,text){const item=document.createElementNS(namespace,tag);for(const [name,value] of Object.entries(attributes))item.setAttribute(name,String(value));if(text)item.textContent=text;svg.append(item);return item;}
  const diameter=study.magnet.diameter_m*1000,height=study.magnet.height_m*1000,scale=20;
  part('rect',{x:200-diameter*scale/2,y:180,width:diameter*scale,height:height*scale,class:'hardware-field-magnet'});part('text',{x:200,y:180+height*scale/2+5,'text-anchor':'middle'},diameter+' × '+height+' mm');
  part('line',{x1:35,y1:240,x2:35,y2:45,class:'hardware-field-axis'});part('text',{x:24,y:32},'+Z');part('text',{x:200,y:272,'text-anchor':'middle'},'Cylinder cross-section · XZ');
  const gapLine=part('line',{class:'hardware-field-gap'}),observer=part('circle',{r:5,class:'hardware-field-observer'}),observerLabel=part('text',{}),gapLabel=part('text',{});geometry.append(svg,parameters);
  const difference=study.validation.maximumSurfaceDifference_T*1e9;
  method.append(propertyList([['Engine',study.engine.name+' '+study.engine.version],['Surface comparisons',study.validation.surfaceCases],['Closed-form checks',study.validation.axisClosedFormCases],['Numeric difference',difference<.001?'< 0.001 nT':difference.toPrecision(3)+' nT'],['Wrong units / direction','Rejected']]),node('p','Surface integral, quadrature refinement, closed-form axis, far-field and coordinate comparisons.'));
  const sources=node('div',undefined,'hardware-field-sources');for(const [label,file] of [['Save calculated study','magnetic-study.json'],['Save Dotdog map','workbench-map.json']]){const source=node('a',label);source.href='/hardware/'+file;source.download=file;sources.append(source);}method.append(sources);
  const mode=viewSwitcher({label:'Field study details',items:[['geometry','Geometry'],['method','Validation']],onChange:value=>{geometry.hidden=value!=='geometry';method.hidden=value!=='method';}});method.hidden=true;inspector.append(mode,geometry,method,node('p','Field prediction only. No AS5600 angle, filtering or hardware acceptance.','hardware-field-limit'));
  function update(){
    root.dataset.scenario=selected.id;const angle=study.angles_deg[index],values=selected.field_T[index];values.forEach((value,i)=>outputs[i].textContent=Math.abs(value)<.0000005?'0.000':(value*1000).toFixed(3));outputs[3].textContent=(Math.hypot(...values)*1000).toFixed(3);positionLabel.value=angle+'°';positionLabel.textContent=positionLabel.value;position.setAttribute('aria-valuetext',angle+' degrees');chart.setAngle(angle);
    for(const [id,button] of buttons){const pressed=String(id===selected.id);button.setAttribute('aria-pressed',pressed);void button.updateComplete.then(()=>button.shadowRoot.querySelector('button')?.setAttribute('aria-pressed',pressed));}
    const x=200+selected.offset_m*1000*scale,y=180-selected.gap_m*1000*scale;for(const [name,value] of Object.entries({cx:x,cy:y}))observer.setAttribute(name,value);for(const [name,value] of Object.entries({x1:x,y1:180,x2:x,y2:y}))gapLine.setAttribute(name,value);observerLabel.setAttribute('x',x+12);observerLabel.setAttribute('y',y-12);observerLabel.textContent='Observer';gapLabel.setAttribute('x',x+12);gapLabel.setAttribute('y',(180+y)/2+5);gapLabel.textContent=selected.gap_m*1000+' mm gap';
    parameters.replaceChildren(propertyList([['Polarization',study.magnet.polarization_T+' T · uniform'],['Lateral offset',selected.offset_m*1000+' mm'],['Observer tilt',selected.observer_tilt_deg+'° about Y']]));
  }
  update();return {root,dispose(){chart.dispose();}};
}

export function mountMagneticStudy(){
  const root=node('div',undefined,'hardware-field-loading');let view=null,sequence=0,disposed=false;
  async function load(){const attempt=++sequence;root.replaceChildren(node('p','Loading calculated field study…'));try{const response=await fetch('/hardware/magnetic-study.json',{cache:'no-store',signal:AbortSignal.timeout(10000)});if(!response.ok)throw new Error('Calculated field study unavailable.');const study=await response.json();if(disposed||attempt!==sequence)return;view?.dispose();view=magneticFieldStudy({study});root.replaceChildren(view.root);}catch(error){if(disposed||attempt!==sequence)return;const retry=commandButton('Retry',{kind:'secondary'});retry.addEventListener('click',()=>void load());root.replaceChildren(node('h2','Field study unavailable'),node('p',error.message),retry);}}
  void load();return {root,dispose(){disposed=true;sequence++;view?.dispose();}};
}
