import * as THREE from 'three';

export function visibleMachineBounds(root){
  root.updateMatrixWorld(true);const bounds=new THREE.Box3();
  root.traverseVisible(item=>{if(item.isMesh){item.geometry.computeBoundingBox();bounds.union(item.geometry.boundingBox.clone().applyMatrix4(item.matrixWorld));}});
  return bounds;
}

export function fitMachineCamera(camera,orbit,bounds,direction,padding=1.08){
  const center=bounds.getCenter(new THREE.Vector3()),radius=bounds.getSize(new THREE.Vector3()).length()/2,forward=direction.clone().normalize(),right=new THREE.Vector3().crossVectors(camera.up,forward).normalize(),up=new THREE.Vector3().crossVectors(forward,right),tan=Math.tan(THREE.MathUtils.degToRad(camera.fov/2));
  let distance=0;
  for(const x of [bounds.min.x,bounds.max.x])for(const y of [bounds.min.y,bounds.max.y])for(const z of [bounds.min.z,bounds.max.z]){const point=new THREE.Vector3(x,y,z).sub(center),depth=point.dot(forward);distance=Math.max(distance,Math.abs(point.dot(right))/(tan*camera.aspect)+depth,Math.abs(point.dot(up))/tan+depth);}
  distance*=padding;camera.near=Math.max(.5,distance-radius*2);camera.far=distance+radius*3;camera.updateProjectionMatrix();orbit.target.copy(center);camera.position.copy(center).addScaledVector(forward,distance);orbit.update();
}
