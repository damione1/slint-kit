// SPDX-License-Identifier: GPL-3.0-or-later
// Eleventy may recompute shared pages; unchanged output bytes keep their mtime.
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
const run = (command, args, extra = {}) => {
  const result = spawnSync(command, args, {stdio:'inherit', ...extra});
  if (result.status !== 0) process.exit(result.status || 1);
};
const staging = '.generated/site-stage';
run('node',['scripts/check-migration.mjs']);
run('python3',['scripts/pack-starters.py']);
fs.rmSync(staging,{recursive:true,force:true});
run('npx',['--no-install','eleventy','--quiet'],{env:{...process.env,DOCS_SITE_STAGE:staging}});
const desired = new Set();
function synchronize(directory, relative='') {
  for(const entry of fs.readdirSync(directory,{withFileTypes:true})) {
    const name = path.join(relative,entry.name);
    if(entry.isDirectory()) { synchronize(path.join(directory,entry.name),name); continue; }
    const target = path.join('_site',name), content=fs.readFileSync(path.join(directory,entry.name));
    desired.add(name);
    if(fs.existsSync(target) && fs.readFileSync(target).equals(content)) continue;
    fs.mkdirSync(path.dirname(target),{recursive:true}); fs.writeFileSync(target,content);
  }
}
function prune(directory,relative='') {
  for(const entry of fs.readdirSync(directory,{withFileTypes:true})) {
    const name=path.join(relative,entry.name), target=path.join(directory,entry.name);
    if(entry.isDirectory()) { prune(target,name); if(fs.readdirSync(target).length===0)fs.rmdirSync(target); }
    else if(!desired.has(name))fs.unlinkSync(target);
  }
}
synchronize(staging); prune('_site');
fs.rmSync(staging,{recursive:true,force:true});
run('node',['scripts/check.mjs']);
