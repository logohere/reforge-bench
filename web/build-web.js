import {createHash} from 'node:crypto';
import {access,readFile,rm,writeFile} from 'node:fs/promises';

const entrypoints=['hardware/plots.css','hardware/hardware.js','hardware-scene.js','ui-components.js','digest.js','workspace-router.js','scope-worker.js','pulse-plot.js','field-plot.js','source-excerpt.js','guides/guides.js'];
await rm(new URL('./dist',import.meta.url),{recursive:true,force:true});
const result=await Bun.build({entrypoints,outdir:'dist',target:'browser',format:'esm',splitting:true,minify:true,conditions:['browser','production'],define:{'process.env.NODE_ENV':'"production"'},naming:{entry:'[name].[ext]'},external:['/dist/*'],plugins:[{name:'packaged-root',setup(build){build.onResolve({filter:/^\/(?!dist\/)/},({path})=>({path:process.cwd()+path}));}}]});
if(!result.success){for(const log of result.logs)console.error(log);process.exit(1);}
for(const page of ['hardware/index.html','lab/index.html','guides/index.html','help/index.html']){
  const html=await readFile(page,'utf8');
  for(const [,url] of html.matchAll(/(?:href|src)="(\/[^"?#]+)(?:\?[^"#]*)?"/g)){
    if(url.endsWith('/'))continue;
    await access('.'+url);
  }
}
const hash=createHash('sha256');
const resources=JSON.parse(await readFile('resources.json','utf8'));
for(const name of [...resources,'resources.json','package.json','bun.lock','build-web.js'].sort()){
  if(typeof name!=='string'||name.startsWith('/')||name.split('/').includes('..'))throw Error('Invalid resource manifest');
  hash.update(name).update(await readFile(name));
}
for(const output of result.outputs.sort((a,b)=>a.path.localeCompare(b.path)))hash.update(new Uint8Array(await output.arrayBuffer()));
await writeFile('build.json',JSON.stringify({id:hash.digest('hex').slice(0,12),scope:'Reforge UI resources and bundles'})+'\n');
console.log(`Built ${result.outputs.length} hardware UI resources`);
