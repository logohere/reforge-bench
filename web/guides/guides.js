import {mountShell,element as node,commandButton} from '../ui/components.js';
import {codeCard} from '../process/source-card.js';
mountShell({hardwareOnly:document.body.dataset.hardwareOnly==='true'});
const article=document.getElementById('guide-article'),topics=document.getElementById('guide-topics'),outline=document.getElementById('guide-outline'),search=document.getElementById('guide-search'),results=document.getElementById('guide-results');
let data;
function link(label,href){const a=node('a',label);a.href=href;if(/^https?:/.test(href)){a.target='_blank';a.rel='noopener noreferrer';}return a;}
function renderTopics(){
  const term=search.value.trim().toLowerCase(),current=location.hash.slice(1).split('/')[0]||data.topics[0].id;let count=0;topics.replaceChildren();
  for(const group of new Set(data.topics.map(t=>t.group))){
    const matches=data.topics.filter(t=>t.group===group&&JSON.stringify(t).toLowerCase().includes(term));if(!matches.length)continue;
    topics.append(node('h2',group));
    for(const topic of matches){const a=link(topic.title,'#'+topic.id);if(topic.id===current)a.setAttribute('aria-current','page');topics.append(a);count++;}
  }
  results.textContent=term?(count?`${count} matching ${count===1?'topic':'topics'}`:'No matching topics. Try “trigger”, “repair” or “probe”.'):'';
}
function showTopic({focus=false}={}){
  const [requested,sectionId]=location.hash.slice(1).split('/'),index=data.topics.findIndex(t=>t.id===requested),topic=data.topics[index<0?0:index],activeIndex=data.topics.indexOf(topic);
  if(requested&&index<0){article.replaceChildren(node('h1','Guide not found'),node('p','Choose a topic from the guide contents.'),link('Setup checklist','#start'));outline.replaceChildren();renderTopics();return;}
  document.title=topic.title+' · Reforge Guides';article.replaceChildren();outline.replaceChildren(node('strong','On this page'));
  const breadcrumb=node('nav',undefined,'guides-breadcrumb');breadcrumb.setAttribute('aria-label','Breadcrumb');breadcrumb.append(link('Guides','#'+data.topics[0].id),node('span','/'),node('span',topic.group));
  article.append(breadcrumb,node('h1',topic.title),node('p',topic.summary,'guides-summary'),node('p',data.scope,'guides-updated'));
  if(topic.prerequisites){const prerequisites=node('ul',undefined,'guides-prerequisites');for(const text of topic.prerequisites)prerequisites.append(node('li',text));article.append(prerequisites);}
  for(const section of topic.sections){
    const part=node('section');part.id='section-'+section.id;const heading=node('h2',section.title);part.append(heading);
    outline.append(link(section.title,'#'+topic.id+'/'+section.id));
    for(const p of section.paragraphs||[])part.append(node('p',p));
    if(section.flow){const flow=node('ol',undefined,'guides-flow');flow.setAttribute('aria-label',section.title+' connection path');for(const label of section.flow)flow.append(node('li',label));part.append(flow);}
    if(section.cards){const cards=node('div',undefined,'guides-cards');for(const card of section.cards){const a=link('',card.href);a.append(node('strong',card.title),node('span',card.description));cards.append(a);}part.append(cards);}
    if(section.callout){const callout=node('aside',undefined,'guides-callout');callout.dataset.kind=section.callout.kind||'info';callout.setAttribute('role','note');callout.append(node('strong',section.callout.title),node('p',section.callout.text));part.append(callout);}
    for(const example of section.commands||[]){const code=codeCard(example.title,example.code,example.language||'Terminal',1,'text'),copy=commandButton('Copy',{kind:'ghost',size:'sm'});copy.setAttribute('aria-label','Copy '+example.title);copy.addEventListener('click',async()=>{try{await navigator.clipboard.writeText(example.code);copy.textContent='Copied';}catch{copy.textContent='Select code to copy';}});code.querySelector('.source-card-heading').append(copy);part.append(code);}
    if(section.steps){const list=node('ol');for(const step of section.steps)list.append(node('li',step));part.append(list);}
    if(section.table){const wrap=node('div',undefined,'guides-table'),table=node('table'),caption=node('caption',section.title),head=node('thead'),row=node('tr'),body=node('tbody');for(const text of section.table.headers){const cell=node('th',text);cell.scope='col';row.append(cell);}head.append(row);for(const cells of section.table.rows){const tr=node('tr');cells.forEach((text,i)=>{const td=node(i?'td':'th',text);if(!i)td.scope='row';tr.append(td);});body.append(tr);}table.append(caption,head,body);wrap.append(table);wrap.tabIndex=0;wrap.setAttribute('role','region');wrap.setAttribute('aria-label',section.title+' table');part.append(wrap);}
    for(const p of section.paragraphsAfter||[])part.append(node('p',p));
    if(section.links){const list=node('ul',undefined,'guides-links');for(const [label,href] of section.links){const li=node('li');li.append(link(label,href));list.append(li);}part.append(list);}
    article.append(part);
  }
  const footer=node('nav',undefined,'guides-pagination');footer.setAttribute('aria-label','Adjacent guides');if(activeIndex>0)footer.append(link('← '+data.topics[activeIndex-1].title,'#'+data.topics[activeIndex-1].id));if(activeIndex<data.topics.length-1)footer.append(link('Next: '+data.topics[activeIndex+1].title+' →','#'+data.topics[activeIndex+1].id));article.append(footer);
  article.scrollTop=0;renderTopics();
  if(sectionId){const target=[...article.querySelectorAll('section')].find(s=>s.id==='section-'+sectionId);target?.scrollIntoView({block:'start'});}
  if(focus)article.focus({preventScroll:true});
}
search.addEventListener('input',()=>{if(data)renderTopics();});
window.addEventListener('hashchange',()=>{if(article.isConnected&&data)showTopic({focus:true});});
try{const response=await fetch('/guides/guides.json');if(!response.ok)throw new Error('Setup topics could not be loaded.');data=await response.json();showTopic();}catch(error){article.replaceChildren(node('h1','Guides unavailable'),node('p',error.message),node('p','Reload this page or read docs/your-project.md in the repository.'));article.setAttribute('role','alert');}
