import {parser} from '@lezer/cpp';
import {highlightTree,tagHighlighter,tags} from '@lezer/highlight';

const highlighter=tagHighlighter([
  {tag:tags.keyword,class:'source-keyword'},
  {tag:tags.typeName,class:'source-type'},
  {tag:[tags.number,tags.bool],class:'source-number'},
  {tag:tags.string,class:'source-string'},
  {tag:tags.comment,class:'source-comment'}
]);

export function sourceTokens(text,language){
  const tokens=[];
  if(language==='c')highlightTree(parser.parse(text),highlighter,(from,to,className)=>tokens.push({from,to,className}));
  return tokens;
}

function highlightedRange(text,tokens,start,end){
  const code=document.createElement('code');code.className='source-code';let position=start;
  for(const token of tokens){
    const from=Math.max(start,token.from),to=Math.min(end,token.to);
    code.append(document.createTextNode(text.slice(position,from)));
    const span=document.createElement('span');span.className=token.className;span.textContent=text.slice(from,to);code.append(span);position=to;
  }
  code.append(document.createTextNode(text.slice(position,end)));return code;
}

export function sourceExcerpt(text,language){
  return highlightedRange(text,sourceTokens(text,language),0,text.length);
}

export function sourceLines(text,language){
  const tokens=sourceTokens(text,language),lines=[];let offset=0,first=0;
  for(const line of text.split('\n')){
    const end=offset+line.length;while(first<tokens.length&&tokens[first].to<=offset)first++;
    let last=first;while(last<tokens.length&&tokens[last].from<end)last++;
    lines.push(highlightedRange(text,tokens.slice(first,last),offset,end));offset=end+1;
  }
  return lines;
}
