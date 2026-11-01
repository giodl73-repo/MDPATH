"""Build the bounded MDPATH demo, native references and inspectable source custody."""
import argparse, hashlib, html, json, os, shutil, subprocess, zipfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]

def capture(*args):return subprocess.check_output(args,cwd=ROOT).decode('utf-8').strip()
def run(*args):subprocess.run(args,cwd=ROOT,check=True)
def pin(path):return {'bytes':path.stat().st_size,'sha256':hashlib.sha256(path.read_bytes()).hexdigest()}

def main():
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('--output',type=Path,default=Path('dist/MDPATH'));a=p.parse_args()
 output=(ROOT/a.output).resolve();dist=(ROOT/'dist').resolve()
 if output==dist or not output.is_relative_to(dist):raise ValueError('Output must be a child of dist')
 if capture('wasm-bindgen','--version')!='wasm-bindgen 0.2.127':raise ValueError('Use wasm-bindgen 0.2.127')
 run('cargo','+1.95.0','build','--locked','--release','--target','wasm32-unknown-unknown','--manifest-path','web/adapter/Cargo.toml','--lib','--features','wasm')
 run('cargo','+1.95.0','build','--locked','--manifest-path','web/adapter/Cargo.toml','--bin','mdpath-inspect')
 metadata=json.loads(capture('cargo','+1.95.0','metadata','--manifest-path','web/adapter/Cargo.toml','--locked','--format-version','1','--filter-platform','wasm32-unknown-unknown','--features','wasm'))
 if output.exists():
  def linked(path):return path.is_symlink() or bool(getattr(path.lstat(),'st_file_attributes',0)&1024)
  if linked(output) or any(linked(p) for p in output.rglob('*')):raise ValueError('Linked output refused')
  shutil.rmtree(output)
 output.mkdir(parents=True)
 target=Path(metadata['target_directory']);run('wasm-bindgen',str(target/'wasm32-unknown-unknown/release/mdpath_web.wasm'),'--target','web','--out-dir',str(output/'pkg'))
 binary=target/'debug'/('mdpath-inspect.exe' if os.name=='nt' else 'mdpath-inspect')
 examples=json.loads(capture(str(binary),'--examples'));sample=examples['named']
 page=(ROOT/'web/index.html').read_text(encoding='utf-8')
 page=page.replace('{{MARKDOWN}}',html.escape(sample['markdown'])).replace('{{URI}}',html.escape(sample['uri'],quote=True))
 page=page.replace('{{EXAMPLES}}',json.dumps(examples,ensure_ascii=False).replace('<','\\u003c'))
 (output/'index.html').write_text(page,encoding='utf-8')
 for name in ('app.js','worker.js','style.css'):shutil.copyfile(ROOT/'web'/name,output/name)
 for name in ('LICENSE','BUILD.md'):shutil.copyfile(ROOT/name,output/name)
 shutil.copytree(ROOT/'tools/licenses/rust',output/'rust')
 (output/'examples.json').write_text(json.dumps(examples,indent=2)+'\n',encoding='utf-8')
 # Include notices for the portable adapter's actual resolved dependency closure,
 # including build-time proc macros, without adding consumer dependencies.
 packages={p['id']:p for p in metadata['packages']};nodes={n['id']:n for n in metadata['resolve']['nodes']}
 first=next(p['id'] for p in metadata['packages'] if p['name']=='mdpath-web');pending=[first];visited=set();notices=[];dependencies=[]
 while pending:
  current=pending.pop()
  if current in visited:continue
  visited.add(current);pending.extend(nodes[current]['dependencies']);pkg=packages[current]
  if pkg['source'] is None:continue
  base=Path(pkg['manifest_path']).parent
  files=sorted({f for pattern in ('LICENSE*','COPYRIGHT*','NOTICE*') for f in base.glob(pattern) if f.is_file()})
  if not files:raise ValueError('No license notice found for '+pkg['name'])
  dependencies.append({'name':pkg['name'],'version':pkg['version'],'license':pkg['license']})
  notices.append('\n\n===== '+pkg['name']+' '+pkg['version']+' ('+str(pkg['license'])+') =====\n')
  for f in files:notices.append('\n--- '+f.name+' ---\n'+f.read_text(encoding='utf-8'))
 notice='MDPATH portable adapter dependency closure, including build-time proc macros.\nThe workbench calls the in-memory resolver; native filesystem entry points are not used by the browser. Original licenses follow.\nRust standard-library notices: rust/COPYRIGHT-library.html and rust/licenses/.\n'+''.join(notices)
 (output/'THIRD-PARTY-NOTICES.txt').write_text(notice,encoding='utf-8')
 dirty=bool(capture('git','status','--porcelain','--untracked-files=all'));commit=capture('git','rev-parse','HEAD')
 tracked=capture('git','ls-files','-z').split('\0')
 if dirty:
  with zipfile.ZipFile(output/'source.zip','w',compression=zipfile.ZIP_DEFLATED) as archive:
   for name in tracked:
    path=ROOT/name
    if path.is_file():
     if not path.resolve().is_relative_to(ROOT):raise ValueError('External source path refused')
     archive.write(path,name)
 else:run('git','archive','--format=zip','--output='+str(output/'source.zip'),'HEAD')
 with zipfile.ZipFile(output/'source.zip') as archive:
  for name in ('web/adapter/src/lib.rs','src/resolver.rs','src/uri.rs','web/adapter/Cargo.lock','Cargo.lock','BUILD.md'):
   expected=(ROOT/name).read_bytes() if dirty else subprocess.check_output(['git','show',commit+':'+name],cwd=ROOT)
   if archive.read(name)!=expected:raise ValueError('Source ZIP differs from build source: '+name)
 for name in ('web/adapter/src/lib.rs','src/resolver.rs','src/uri.rs','web/adapter/Cargo.lock','Cargo.lock'):
  dest=output/'source'/name;dest.parent.mkdir(parents=True,exist_ok=True)
  dest.write_bytes((ROOT/name).read_bytes() if dirty else subprocess.check_output(['git','show',commit+':'+name],cwd=ROOT))
 provenance={'schema':'mdpath.pages-build.v1','commit':commit,'dirty':dirty,'source':'https://github.com/giodl73-repo/MDPATH',
   'rust':capture('rustc','+1.95.0','--version'),'bindings':capture('wasm-bindgen','--version'),'dependencies':sorted(dependencies,key=lambda p:p['name']),
   'example_requests':7,'delivery_budget_bytes':5_000_000}
 (output/'provenance.json').write_text(json.dumps(provenance,indent=2)+'\n',encoding='utf-8');(output/'.nojekyll').touch()
 cache=ROOT/'.cache';cache.mkdir(exist_ok=True);binary=target/'debug'/('mdpath-inspect.exe' if os.name=='nt' else 'mdpath-inspect')
 cases=[]
 def case(name,record):
  path=cache/'request.json';path.write_text(record,encoding='utf-8',newline='')
  canonical=capture(str(binary),str(path));json.loads(canonical);cases.append({'name':name,'input':record,'canonical':canonical})
 for name,pair in examples.items():case(name,json.dumps(pair))
 def altered(name,**changes):case(name,json.dumps({**sample,**changes},ensure_ascii=False))
 altered('invalid-uri',uri='https://example.invalid/guide.md')
 altered('missing-section',uri='md://guide.md#missing')
 altered('missing-label',uri='md://guide.md#address-lab:figure:absent')
 altered('wrong-document',path='other.md')
 altered('absolute-path',path='/guide.md')
 altered('over-bytes',markdown=' '*131073)
 altered('over-lines',markdown='\n'*8193)
 altered('over-uri',uri='x'*4097)
 altered('portable-number',uri='md://guide.md#address-lab:figure:4294967296')
 altered('portable-subselector',uri='md://guide.md#address-lab/measurements:table:0[row=4294967296]')
 altered('portable-query',uri='md://guide.md#address-lab/measurements:table:0?skip=4294967296')
 altered('max-portable-selector',uri='md://guide.md#address-lab:figure:4294967295')
 altered('max-portable-subselector',uri='md://guide.md#address-lab/measurements:table:0[row=4294967295]')
 altered('max-portable-query',uri='md://guide.md#address-lab/measurements:table:0?skip=4294967295')
 altered('inert-unicode',markdown='# Notes\n\nCafé 🌿 <img src=x onerror=alert(1)>\n',uri='md://guide.md#notes:text:0')
 case('invalid-envelope','{}');case('invalid-json','[broken');case('over-envelope',' '*1048577)
 (cache/'native-references.json').write_text(json.dumps(cases,indent=2)+'\n',encoding='utf-8')
 files={p.relative_to(output).as_posix():pin(p) for p in sorted(output.rglob('*')) if p.is_file()}
 size=sum(p['bytes'] for p in files.values())
 if size>=5_000_000:raise ValueError('Static demo exceeds the 5 MB delivery budget')
 (cache/'site-inventory.json').write_text(json.dumps({'commit':commit,'dirty':dirty,'bytes':size,'files':files},indent=2)+'\n')
 print(json.dumps({'output':str(output),'bytes':size,'files':len(files),'native_reference_cases':len(cases),'dirty':dirty}))

if __name__=='__main__':main()
