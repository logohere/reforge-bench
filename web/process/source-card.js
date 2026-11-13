import {element as node} from '/ui/components.js';
import {sourceLines} from '/dist/source-excerpt.js';

export function codeCard(title,source,detail,start=1,language=/\.[ch]\b|ghidra|pseudocode|reconstructed source/i.test(title+' '+detail)?'c':'text'){
  const root=node('section',undefined,'source-card'),heading=node('div',undefined,'source-card-heading'),lines=node('ol',undefined,'code-lines');
  heading.append(node('span',title),node('span',detail));lines.start=start;lines.style.setProperty('--line-start',start-1);
  const rendered=sourceLines(source,language),text=source.split('\n');
  for(let index=0;index<rendered.length;index++){
    const item=node('li');item.append(rendered[index]);
    if(/ioctl|system\(|FUN_000b1a04|DAT_002eb404|logo\.bmp|RECORD_STRIDE|OP_PREVIEW|OP_EXECUTE|firmware_dispatch|self.call\(|exchange\(/.test(text[index]))item.className='highlight';
    lines.append(item);if(index<rendered.length-1)lines.append(document.createTextNode('\n'));
  }
  root.append(heading,lines);return root;
}

export function exportedCodeCard(title,source,{identity='',language='c'}={}){
  const lines=source.split('\n'),marker='// pseudocode: provider-specific; not original source';
  const boundary=lines[0].replace(/\r$/,'')==='// rea decompile_all'?lines.findIndex(line=>line.replace(/\r$/,'')===marker):-1;
  let start=boundary>=0&&lines.slice(0,boundary).every(line=>!line.trim()||line.startsWith('//'))?boundary+1:0;
  if(start){while(start<lines.length&&!lines[start].trim())start++;if(start===lines.length)start=0;}
  const root=node('div',undefined,'source-export');root.append(codeCard(title,lines.slice(start).join('\n'),'Decompiler output · not original source',start+1,language));
  if(start||identity){const full=node('details');full.append(node('summary',start?'Full export and provenance':'Source identity'));if(start)full.append(codeCard('Retained export',source,'Exact text including provider metadata',1,language));if(identity)full.append(node('p','SHA-256 · '+identity,'provenance'));root.append(full);}
  return root;
}
