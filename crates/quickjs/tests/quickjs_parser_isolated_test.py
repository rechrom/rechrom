#!/usr/bin/env python3
"""Compile the parser oracle without other concurrently edited staged groups."""
from pathlib import Path
import os,re,subprocess,tempfile,sys
root=Path(__file__).resolve().parents[1]
tmp=Path(tempfile.mkdtemp(prefix='quickjs-parser-rustc-'))
def absolutize(source,parent):
 return re.sub(r'(include!\(|include_str!\(|#\[path\s*=\s*)"([^"]+)"',lambda m:m[1]+'"'+str((parent/m[2]).resolve())+'"',source)
lib=root.joinpath('src/lib.rs').read_text()
lib=re.sub(r'pub mod (\w+);',lambda m:('#[path = "'+str(root/'src'/f'{m[1]}.rs')+'"] pub mod '+m[1]+';')if m[1]!='quickjs' else '__QUICKJS__',lib)
q=root.joinpath('src/quickjs.rs').read_text().split('#[cfg(test)]')[0]
a=root.joinpath('tests/quickjs_atoms_differential.rs').read_text().split('include!("quickjs_classes_differential.rs");')[0]+'\ninclude!("quickjs_parser_differential.rs");\n'
q=absolutize(q,root/'src')+'\n#[cfg(test)] mod atom_tests {\n'+absolutize(a,root/'tests')+'\n}\n'
lib=lib.replace('__QUICKJS__','pub mod quickjs {\n'+q+'\n}')
src=tmp/'harness.rs';src.write_text(lib);exe=tmp/'harness'
env=os.environ.copy();env['CARGO_MANIFEST_DIR']=str(root)
args=['rustc','+1.98.0','--edition','2021','--test','-C','opt-level='+('0' if '--debug' in sys.argv else '3'),'-C','debuginfo=0','-C','codegen-units=1','-A','warnings','--crate-name','quickjs_parser_harness']
if '--no-default-features' not in sys.argv:
 args.extend(['--cfg','feature="short-opcodes"','--cfg','feature="atomics"'])
if '--full-source' in sys.argv:args.extend(['--cfg','parser_full_source'])
if '--typecheck' in sys.argv:args.extend(['--emit','metadata'])
args.extend([str(src),'-o',str(exe)])
try:
 subprocess.run(args,env=env,check=True)
 if '--typecheck' not in sys.argv:subprocess.run([str(exe),'official_full_c_javascript_lexer','--nocapture'],env=env,check=True)
finally:
 import shutil
 if '--keep' in sys.argv:print(tmp,flush=True)
 else:shutil.rmtree(tmp)
