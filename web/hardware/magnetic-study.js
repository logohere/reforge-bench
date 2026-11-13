export function checkedMagneticStudy(study){
  const fail=()=>{throw new Error('Magnetic study is missing, invalid or not independently checked.');};
  if(study?.schema!==1||study.kind!=='calculated-field-study'||study.engine?.name!=='Magpylib'||study.engine.version!=='5.2.3'||study.validation?.passed!==true)fail();
  if(study.units?.length!=='m'||study.units.field!=='T'||study.units.polarization!=='T'||study.units.rotation!=='deg')fail();
  if(!study.basis||!Array.isArray(study.limits)||!study.limits.length||!study.sourceSha256||Object.keys(study.sourceSha256).length<5||!Object.values(study.sourceSha256).every(value=>/^[a-f0-9]{64}$/.test(value)))fail();
  const angles=study.angles_deg;if(!Array.isArray(angles)||angles.length<3||angles.length>361||angles[0]!==0||angles.at(-1)!==360||!angles.includes(90)||angles.some((value,i)=>!Number.isFinite(value)||i&&value<=angles[i-1]))fail();
  const magnet=study.magnet;if(!magnet||['diameter_m','height_m','polarization_T'].some(key=>!Number.isFinite(magnet[key])||magnet[key]<=0))fail();
  if(!Array.isArray(study.scenarios)||study.scenarios.length!==4||new Set(study.scenarios.map(item=>item.id)).size!==4)fail();
  for(const item of study.scenarios){
    if(!item.id||!item.label||!Number.isFinite(item.gap_m)||item.gap_m<=0||!Number.isFinite(item.offset_m)||!Number.isFinite(item.observer_tilt_deg)||!Array.isArray(item.observer_m)||item.observer_m.length!==3||!item.observer_m.every(Number.isFinite))fail();
    if(Math.abs(item.observer_m[2]-magnet.height_m/2-item.gap_m)>1e-12||item.observer_m[0]!==item.offset_m||item.observer_m[1]!==0)fail();
    if(!Array.isArray(item.field_T)||item.field_T.length!==angles.length||item.field_T.some(row=>!Array.isArray(row)||row.length!==3||!row.every(Number.isFinite)))fail();
    if(item.field_T[0].some((value,axis)=>Math.abs(value-item.field_T.at(-1)[axis])>1e-9))fail();
    const z=item.observer_m[2],radius=magnet.diameter_m/2,half=magnet.height_m/2,axial=-magnet.polarization_T/4*((z+half)/Math.hypot(radius,z+half)-(z-half)/Math.hypot(radius,z-half)),tilt=item.observer_tilt_deg*Math.PI/180;
    for(let i=0;i<angles.length;i++){
      const angle=angles[i]*Math.PI/180,expected=item.offset_m===0?[axial*Math.cos(angle)*Math.cos(tilt),axial*Math.sin(angle),axial*Math.cos(angle)*Math.sin(tilt)]:item.field_T[0].map((value,axis)=>value*Math.cos(angle)+item.field_T[angles.indexOf(90)][axis]*Math.sin(angle));
      if(item.field_T[i].some((value,axis)=>Math.abs(value-expected[axis])>1e-9+1e-6*Math.hypot(...expected)))fail();
    }
  }
  const validation=study.validation;
  if(!Number.isSafeInteger(validation.surfaceCases)||validation.surfaceCases<40||!Number.isSafeInteger(validation.axisClosedFormCases)||validation.axisClosedFormCases<5||!Number.isSafeInteger(validation.observerBasisCases)||validation.observerBasisCases<3||!validation.negativeControls?.millimetersAsMetersRejected||!validation.negativeControls.wrongMagnetizationDirectionRejected||!validation.negativeControls.degreesAsRadiansRejected)fail();
  return study;
}
