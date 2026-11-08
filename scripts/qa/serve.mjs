// Foreground static server for deterministic browser tests (Astro preview can daemonize).
import {createServer} from 'node:http';
import {readFile,stat} from 'node:fs/promises';
import {resolve,extname,sep} from 'node:path';
const root=resolve(import.meta.dirname,'../../dist');
const base=(process.env.BASE ?? '/learning-guide').replace(/\/$/,'')+'/';
const mime={'.html':'text/html','.js':'text/javascript','.css':'text/css','.json':'application/json','.wasm':'application/wasm','.svg':'image/svg+xml','.woff2':'font/woff2'};
createServer(async(req,res)=>{
  try {
    const url=new URL(req.url,'http://localhost');
    if (!url.pathname.startsWith(base)) {res.writeHead(404).end();return;}
    let file=resolve(root,decodeURIComponent(url.pathname.slice(base.length)) || '.');
    if (file!==root && !file.startsWith(root+sep)) {res.writeHead(403).end();return;}
    if ((await stat(file)).isDirectory()) file=resolve(file,'index.html');
    res.writeHead(200,{'Content-Type':mime[extname(file)] ?? 'application/octet-stream','Cache-Control':'no-store'});
    res.end(await readFile(file));
  } catch {res.writeHead(404).end('Not found');}
}).listen(4322,'127.0.0.1',()=>console.log('Browser test server: http://127.0.0.1:4322'+base));
