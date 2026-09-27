#!/usr/bin/env python3
"""Bounded process-level transport checks. Every process has an independent engine."""
import os,sys,time,subprocess,pathlib,re,json
root=pathlib.Path(__file__).resolve().parents[1]
mode=sys.argv[1] if len(sys.argv)>1 else 'direct'
count=int(sys.argv[2]) if len(sys.argv)>2 else 2
seconds=int(sys.argv[3]) if len(sys.argv)>3 else 50
out=root/'artifacts'/('v02-network-'+mode+str(count));out.mkdir(exist_ok=True)
for old in out.iterdir():
 if old.is_file():old.unlink()
processes=[];streams=[]
env=dict(os.environ,HK_TEST_DISABLE_MDNS='1')
def start(name,args):
 f=(out/(name+'.log')).open('w');streams.append(f);p=subprocess.Popen(args,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT);processes.append(p);return p

def wait_address(name,circuit=False):
 deadline=time.monotonic()+20
 while time.monotonic()<deadline:
  text=(out/(name+'.log')).read_text()
  for line in text.splitlines():
   if ('LISTEN ' in line or 'RELAY ' in line) and '/tcp/' in line and ('/p2p-circuit/' in line)==circuit:
    return re.sub(r'/ip4/[^/]+','/ip4/127.0.0.1',line.split(' ',1)[1])
  time.sleep(.2)
 raise RuntimeError('No transport address '+name)
try:
 relay=None
 if mode=='relay':
  start('relay',['target/release/hexkeep-relay','4017']);relay=wait_address('relay')
 cmd=['target/release/hexkeep-sim','peer','--id','0','--seconds',str(seconds+12),'--autostart',str(count-1),'--out',str(out/'proof-0.json')]
 if relay:cmd+=['--relay',relay]
 start('peer-0',cmd);address=wait_address('peer-0',bool(relay))
 for i in range(1,count):start('peer-'+str(i),['target/release/hexkeep-sim','peer','--id',str(i),'--seconds',str(seconds),'--connect',address,'--out',str(out/('proof-'+str(i)+'.json'))])
 deadline=time.monotonic()+seconds+25
 for p in processes[(1 if relay else 0):]:p.wait(timeout=max(1,deadline-time.monotonic()))
 checkpoints=[]
 for i in range(count):
  text=(out/('peer-'+str(i)+'.log')).read_text();checkpoints.append(dict(re.findall(r'CHECK (\d+) ([a-f0-9]+)',text)))
 common=set.intersection(*[set(c) for c in checkpoints]);assert len(common)>=10,(common,[len(c)for c in checkpoints])
 for tick in common:assert len({c[tick]for c in checkpoints})==1,('DIVERGENCE',tick)
 if mode=='relay' and count==2:
  for i in range(count):assert json.loads((out/('proof-'+str(i)+'.json')).read_text()).get('complete'),'Duel relay incomplet'
 result=f'PASS {count} independent peers; {len(common)} equal checkpoints; through tick {max(map(int,common))}; {mode}'
 (out/'result.txt').write_text(result+'\n');print(result)
 for i in range(count):
  proof=out/('proof-'+str(i)+'.json')
  if proof.exists() and proof.stat().st_size and json.loads(proof.read_text()).get('complete'):
   with (out/('replay-'+str(i)+'.log')).open('w')as f:subprocess.run(['target/release/hexkeep-sim','replay',str(proof)],cwd=root,check=True,stdout=f,stderr=f)
finally:
 for p in processes:
  if p.poll()is None:p.terminate()
 for f in streams:f.close()
