import {coraComponents} from './cora-model.js';

export function populateCora(group,{THREE,box,cylinder,pcb,metal,black,white,gold,material,color}){
  const shape=new THREE.Shape();shape.moveTo(-50.8,-28.95);shape.lineTo(50.8,-28.95);shape.lineTo(50.8,28.95);shape.lineTo(-50.8,28.95);shape.closePath();
  for(const x of [-47,47])for(const z of [-25,25]){const hole=new THREE.Path();hole.absarc(x,z,1.6,0,Math.PI*2,true);shape.holes.push(hole);}
  const board=new THREE.Mesh(new THREE.ExtrudeGeometry(shape,{depth:1.6,bevelEnabled:false}),pcb);board.rotation.x=-Math.PI/2;group.add(board);
  const stamp=(text,w,d,x,y,z,{size=42,background=null}={})=>{const canvas=document.createElement('canvas');canvas.width=Math.max(256,Math.ceil(w*24));canvas.height=Math.max(64,Math.ceil(d*24));const ctx=canvas.getContext('2d');if(background){ctx.fillStyle=background;ctx.fillRect(0,0,canvas.width,canvas.height);}ctx.fillStyle=color('--rb-color-text').getStyle();ctx.font=size+'px monospace';ctx.textAlign='center';ctx.textBaseline='middle';ctx.fillText(text,canvas.width/2,canvas.height/2);const map=new THREE.CanvasTexture(canvas);map.colorSpace=THREE.SRGBColorSpace;const mesh=new THREE.Mesh(new THREE.PlaneGeometry(w,d),new THREE.MeshBasicMaterial({map,transparent:true,depthWrite:false}));mesh.rotation.x=-Math.PI/2;mesh.position.set(x,y,z);group.add(mesh);};
  for(const [name,x,z,w,d,h,type] of coraComponents){const cx=x+w/2,cz=z+d/2,base=1.6;
    if(name==='RJ45'){
      for(const side of [-1,1]){box(group,[w,.7,d],[cx,base+(side<0?.35:h-.35),cz],metal);box(group,[w,h,.7],[cx,base+h/2,cz+side*(d/2-.35)],metal);}box(group,[.6,h,d],[x+w-.3,base+h/2,cz],black);
      box(group,[.8,3.2,7],[x+w-.8,base+2,cz],black);for(let i=0;i<8;i++)box(group,[7,.35,.4],[x+7,base+h-3,cz+(i-3.5)*1.3],gold);
      for(const side of [-1,1])box(group,[.4,1.3,2],[x-.2,base+h-1.5,cz+side*(d/2-2)],white);group.children.at(-1).userData.port='RJ45';
    }else if(name==='HOST'||name==='J12'){
      const height=name==='J12'?3.2:h,face=name==='J12'?z:z+d,back=name==='J12'?z+d:z;
      for(const side of [-1,1]){box(group,[w,.6,d],[cx,base+(side<0?.3:height-.3),cz],metal);box(group,[.6,height,d],[cx+side*(w/2-.3),base+height/2,cz],metal);}box(group,[w-1,height-1,.5],[cx,base+height/2,back],black);
      box(group,[w-2,1,3],[cx,base+height*.42,face+(name==='J12'?1.5:-1.5)],black);for(let i=0;i<(name==='J12'?5:4);i++)box(group,[name==='J12'?.3:.8,.12,2],[cx+(i-(name==='J12'?2:1.5))*(name==='J12'?.75:2.1),base+height*.42+.6,face+(name==='J12'?1.2:-1.2)],gold);
    }else if(name==='5V'){
      box(group,[w,8,d-3],[cx,5.6,cz+1.5],black,.5);cylinder(group,4.5,d,[cx,6.5,cz],black,'z');cylinder(group,3.2,.35,[cx,6.5,z-.15],metal,'z');cylinder(group,2.6,.4,[cx,6.5,z-.4],black,'z');cylinder(group,.75,2,[cx,6.5,z-.6],metal,'z');
    }else{
      box(group,[w,h,d],[cx,base+h/2,cz],type==='button'?metal:type==='led'?white:type==='jumper'?material(color('--rb-color-info-bg')):black,.25);
      if(type==='button')cylinder(group,1.7,1.2,[cx,h+2.2,cz],black);
      if(name==='J1'||name==='J5')for(let col=0;col<Math.floor(w/2.54);col++)for(const row of [0,1]){box(group,[1.5,.18,1.5],[x+1.4+col*2.54,h+1.7,z+1.1+row*2.54],metal);box(group,[1.05,.2,1.05],[x+1.4+col*2.54,h+1.82,z+1.1+row*2.54],black);}
      if(name==='JA'||name==='JB')for(let row=0;row<6;row++)for(const side of [-1,1]){box(group,[4,.6,.6],[x-2,base+h/2+side*1.27,z+1.15+row*2.54],gold);box(group,[.18,1.2,1.2],[x+w+.1,base+h/2+side*1.27,z+1.15+row*2.54],metal);box(group,[.2,.8,.8],[x+w+.2,base+h/2+side*1.27,z+1.15+row*2.54],black);}
      if(['FTDI','ZYNQ','DDR3L'].includes(name))stamp(name,w-1,d-1,cx,base+h+.03,cz,{size:name==='ZYNQ'?48:28});
    }
  }
  for(const [text,x,z,w,d] of [['CORA Z7',18,-14,27,5],['DIGILENT',13,18,25,4],['JA',39,-10,5,3],['JB',39,12,5,3],['J12 USB',-18.5,-22,12,3],['BTN1',-21.6,20,8,2],['BTN0',-9.6,20,8,2],['LD1',-20.7,15.5,5,2],['LD0',-8.7,15.5,5,2],['PWR',-44,-18,5,2],['DONE',-44,22,6,2]])stamp(text,w,d,x,1.68,z,{size:text==='CORA Z7'?72:36});
  for(const [x,z] of [[-29,-22],[-27,-15],[-26,-8],[-13,-18],[-10,-18],[-7,-18],[-27,7],[-24,7],[-21,7],[-18,7],[-15,7],[-12,7],[-9,7],[-6,7],[-3,7],[0,-11],[4,-11],[8,-11],[23,12],[27,12],[31,12],[34,17],[-28,14],[-27,20]]){box(group,[1.5,.7,.8],[x,1.95,z],black,.1);for(const side of [-1,1])box(group,[.3,.75,.85],[x+side*.6,1.95,z],metal,.04);}
  for(const x of [-47,47])for(const z of [-25,25]){const rim=new THREE.Mesh(new THREE.RingGeometry(1.6,2,24),gold);rim.rotation.x=-Math.PI/2;rim.position.set(x,1.65,z);group.add(rim);}
}
