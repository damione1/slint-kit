// SPDX-License-Identifier: GPL-3.0-or-later
export function normalizeSearch(value) {
  return String(value).replace(/([a-z\d])([A-Z])/g,'$1 $2').normalize('NFD').replace(/\p{M}/gu,'').toLowerCase().replace(/[^\p{L}\p{N}]+/gu,' ').trim();
}
function words(value) { return [...new Set(normalizeSearch(value).split(' ').filter(Boolean))]; }
function distance(a,b,maximum) {
  if(Math.abs(a.length-b.length)>maximum)return maximum+1;
  const rows=[Array.from({length:b.length+1},(_,i)=>i)];
  for(let i=1;i<=a.length;i++){
    const row=[i];
    for(let j=1;j<=b.length;j++){
      row[j]=Math.min(rows[i-1][j]+1,row[j-1]+1,rows[i-1][j-1]+(a[i-1]===b[j-1]?0:1));
      if(i>1&&j>1&&a[i-1]===b[j-2]&&a[i-2]===b[j-1])row[j]=Math.min(row[j],rows[i-2][j-2]+1);
    }
    rows.push(row);
  }
  return rows[a.length][b.length];
}
function wordScore(query,word) {
  if(query===word)return 100;
  if(word.startsWith(query))return 84-Math.min(20,word.length-query.length);
  if(query.length<3)return 0;
  const maximum=query.length>=6?2:1;
  const edits=distance(query,word,maximum);
  if(edits<=maximum)return 65-edits*13;
  if(query.length>=3&&word[0]===query[0]&&word.length<=query.length*2.5){
    let index=0;for(const character of word)if(character===query[index])index++;
    if(index===query.length)return 35-(word.length-query.length);
  }
  return 0;
}
export function prepareSearch(entries) {
  return entries.map(entry=>({...entry,apiIdentity:normalizeSearch(entry.api||'').replaceAll(' ',''),fields:[{weight:6,words:[...words(entry.api||''),normalizeSearch(entry.api||'').replaceAll(' ','')].filter(Boolean)},{weight:4,words:words(entry.title)},{weight:2,words:words(entry.summary)},{weight:1,words:words(entry.text)}]}));
}
export function searchDocuments(entries,query,limit=8) {
  const tokens=words(query).slice(0,8);if(!tokens.length)return [];
  return entries.map(entry=>{
    let score=0;
    for(const token of tokens){
      let best=0;
      for(const field of entry.fields)for(const word of field.words)best=Math.max(best,wordScore(token,word)*field.weight);
      if(!best)return null;
      score+=best;
    }
    const identity=tokens.join('');
    if(entry.apiIdentity&&(entry.apiIdentity===identity||entry.apiIdentity.replace(/^kit/,'')===identity))score+=10000;
    return {entry,score};
  }).filter(Boolean).sort((a,b)=>b.score-a.score||a.entry.title.localeCompare(b.entry.title)).slice(0,limit).map(({entry})=>entry);
}
function excerpt(entry,query) {
  const apiWords=words(entry.api||'');
  const ownApiQuery=entry.api&&words(query).every(token=>apiWords.some(word=>wordScore(token,word)>0)||wordScore(token,entry.apiIdentity)>0);
  let source=(ownApiQuery?entry.summary:entry.text)||entry.summary;
  const token=words(query).find(word=>normalizeSearch(source).includes(word));
  if(!token&&entry.summary)source=entry.summary;
  // Normalization may change string length; use word boundaries in the original
  // prose rather than slicing it with normalized offsets.
  const parts=source.split(/\s+/), index=token?parts.findIndex(word=>normalizeSearch(word).includes(token)):-1;
  const start=Math.max(0,index-8), body=parts.slice(start,start+32).join(' ');
  return (start?'… ':'')+body+(start+32<parts.length?' …':'');
}
export function setupSearch(document,fetchIndex=globalThis.fetch.bind(globalThis)) {
  const trigger=document.querySelector('.doc-search-trigger'), dialog=document.querySelector('#doc-search');
  if(!trigger||!dialog)return;
  const input=dialog.querySelector('input'), list=dialog.querySelector('.doc-search-results'), status=dialog.querySelector('[role="status"]');
  const french=document.documentElement.lang==='fr';let entries,pending,originFocus;
  const copy={loading:french?'Chargement de la recherche…':'Loading search…',empty:french?'Aucun résultat. Essayez un autre mot.':'No results. Try another word.',start:french?'Recherchez un composant, une propriété ou un guide.':'Search for a component, property or guide.',error:french?'Recherche indisponible. Réessayez ou utilisez la navigation.':'Search unavailable. Retry or use the navigation.'};
  async function load(){
    if(entries)return entries;
    pending??=fetchIndex(trigger.dataset.index).then(response=>{if(!response.ok)throw Error('Search index unavailable');return response.json()}).then(index=>{
      if(index.schema!==1||index.locale!==document.documentElement.lang||!Array.isArray(index.entries))throw Error('Invalid search index');
      entries=prepareSearch(index.entries);return entries;
    }).catch(error=>{pending=null;throw error});
    return pending;
  }
  let request=0;
  async function update(){
    const current=++request, query=input.value.trim();list.replaceChildren();
    if(!query){status.textContent=copy.start;return;}
    status.textContent=copy.loading;
    try{
      const data=await load();if(current!==request)return;
      const results=searchDocuments(data,query);
      for(const result of results){
        const item=document.createElement('li'), link=document.createElement('a'), title=document.createElement('strong'), description=document.createElement('span');
        link.href=result.url;title.textContent=result.title;description.textContent=excerpt(result,query);link.append(title,description);item.append(link);list.append(item);
      }
      status.textContent=results.length?(french?`${results.length} résultat${results.length>1?'s':''}`:`${results.length} result${results.length>1?'s':''}`):copy.empty;
    }catch{if(current===request)status.textContent=copy.error;}
  }
  const open=()=>{if(dialog.open)return;originFocus=document.activeElement;dialog.showModal();input.focus();update();};
  trigger.hidden=false;trigger.addEventListener('click',open);
  dialog.querySelector('.doc-search-close').addEventListener('click',()=>dialog.close());
  dialog.addEventListener('click',event=>{if(event.target===dialog){const box=dialog.getBoundingClientRect();if(event.clientX<box.left||event.clientX>box.right||event.clientY<box.top||event.clientY>box.bottom)dialog.close();}});
  dialog.addEventListener('close',()=>{++request;originFocus?.focus();});
  input.addEventListener('input',update);
  dialog.querySelector('form').addEventListener('submit',event=>{event.preventDefault();list.querySelector('a')?.click();});
  dialog.addEventListener('keydown',event=>{
    if(event.isComposing)return;
    if(event.key!=='ArrowDown'&&event.key!=='ArrowUp')return;
    const links=[...list.querySelectorAll('a')];if(!links.length)return;
    event.preventDefault();const index=links.indexOf(document.activeElement), step=event.key==='ArrowDown'?1:-1;
    links[(index===-1?(step===1?0:links.length-1):(index+step+links.length)%links.length)].focus();
  });
  document.addEventListener('keydown',event=>{if(!event.isComposing&&(event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();open();}});
}
if(typeof document!=='undefined')setupSearch(document);
