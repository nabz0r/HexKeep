#!/usr/bin/env python3
import pathlib,subprocess,time,os,re,json,shutil
root=pathlib.Path(__file__).resolve().parents[1];out=root/'artifacts'/'v03-android-ten';out.mkdir(exist_ok=True)
for old in out.iterdir():
 if old.is_file():old.unlink()
adb=os.environ.get('ADB') or shutil.which('adb') or str(pathlib.Path(os.environ.get('ANDROID_HOME',str(pathlib.Path.home()/'Library/Android/sdk')))/'platform-tools/adb');env=dict(os.environ,HK_TEST_DISABLE_MDNS='1');processes=[];files=[]
def start(name,args):
 f=(out/(name+'.log')).open('w');files.append(f);p=subprocess.Popen(args,cwd=root,env=env,stdout=f,stderr=subprocess.STDOUT);processes.append(p);return p
try:
 start('peer-0',['target/release/hexkeep-sim','peer','--id','0','--seconds','110','--autostart','9'])
 deadline=time.monotonic()+15;address=None
 while time.monotonic()<deadline:
  match=re.search(r'LISTEN (/ip4/[^\n]+/tcp/[^\n]+)',(out/'peer-0.log').read_text())
  if match:address=re.sub(r'/ip4/[^/]+','/ip4/127.0.0.1',match.group(1));break
  time.sleep(.2)
 assert address,'no host address'
 for i in range(1,8):start('peer-'+str(i),['target/release/hexkeep-sim','peer','--id',str(i),'--seconds','100','--connect',address])
 android_address=address.replace('/ip4/127.0.0.1','/ip4/10.0.2.2')
 runners=[]
 for serial in ['emulator-5554','emulator-5556']:
  runners.append((serial,start(serial,[adb,'-s',serial,'shell','am','instrument','-w','-e','class','game.hexkeep.NetworkHarness','-e','peerAddress',android_address,'game.hexkeep.dev.test/androidx.test.runner.AndroidJUnitRunner'])))
 for serial,p in runners:
  p.wait(timeout=100);text=(out/(serial+'.log')).read_text();assert 'OK (1 test)' in text,text
  subprocess.run([adb,'-s',serial,'pull','/sdcard/Android/data/game.hexkeep.dev/files/network-checks.jsonl',str(out/(serial+'.jsonl'))],check=True,stdout=subprocess.DEVNULL)
 maps=[]
 for i in range(8):maps.append(dict(re.findall(r'CHECK (\d+) ([a-f0-9]+)',(out/('peer-'+str(i)+'.log')).read_text())))
 for serial,_ in runners:maps.append({str(v['tick']):v['hash']for v in map(json.loads,(out/(serial+'.jsonl')).read_text().splitlines())})
 common=set.intersection(*(set(m)for m in maps));assert len(common)>=40,[len(m)for m in maps]
 for t in common:assert len({m[t]for m in maps})==1,('diverged',t)
 result=f'PASS two Android + eight native peers; {len(common)} matching checkpoints; tick {max(map(int,common))}'
 (out/'result.txt').write_text(result+'\n');print(result)
finally:
 for p in processes:
  if p.poll()is None:p.terminate()
 for f in files:f.close()
