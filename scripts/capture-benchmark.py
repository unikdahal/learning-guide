"""Capture a deliberate local benchmark; no environment values or uploads are recorded."""
import argparse,json,os,platform,subprocess,time
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--question',required=True);p.add_argument('--cache-state',choices=['warm','cold','unspecified'],required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('command',nargs=argparse.REMAINDER);args=p.parse_args()
cmd=args.command[1:] if args.command[:1]==['--'] else args.command
if not cmd:p.error('provide a command after --')
root=Path(__file__).resolve().parents[1];args.output.parent.mkdir(parents=True,exist_ok=True)
raw=args.output.with_suffix('.samples.jsonl');raw.write_text('')
def read_command(command):
    try:return subprocess.check_output(command,cwd=root,text=True,stderr=subprocess.DEVNULL).strip()
    except (OSError,subprocess.CalledProcessError):return None
cpu=None;memory=None
if Path('/proc/cpuinfo').exists():cpu=next((s.split(':',1)[1].strip() for s in Path('/proc/cpuinfo').read_text().splitlines() if s.startswith('model name')),None)
if Path('/proc/meminfo').exists():memory=next((s.split(':',1)[1].strip() for s in Path('/proc/meminfo').read_text().splitlines() if s.startswith('MemTotal:')),None)
env=os.environ.copy();env['QE_BENCH_SAMPLES']=str(raw.resolve());start=time.monotonic()
run=subprocess.run(cmd,cwd=root,env=env,text=True,capture_output=True)
dataset=root/'lab/data/generator.json'
record={'schema':1,'question':args.question,'cache_state':args.cache_state,'command':cmd,'commit':read_command(['git','rev-parse','HEAD']),'dirty':bool(read_command(['git','status','--porcelain'])),'compiler':read_command(['rustc','--version']),'os':platform.platform(),'cpu':cpu,'logical_cpus':os.cpu_count(),'memory':memory,'dataset':json.loads(dataset.read_text()) if dataset.exists() else None,'elapsed_seconds':time.monotonic()-start,'exit_code':run.returncode,'stdout':run.stdout,'stderr':run.stderr,'raw_samples':str(raw),'series':[json.loads(s) for s in raw.read_text().splitlines() if s.strip()]}
args.output.write_text(json.dumps(record,indent=2)+'\n');print(f'Wrote {args.output}; command exit {run.returncode}');raise SystemExit(run.returncode)
