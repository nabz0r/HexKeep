use hk_proto::*;
use std::time::{Duration, Instant};
fn main() {
    if let Err(e) = run() {
        eprintln!("ERREUR: {e}");
        std::process::exit(1)
    }
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let option = |name: &str| {
        args.iter()
            .position(|s| s == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    match args.get(1).map(String::as_str){
 Some("determinism")=>println!("10000 combats complets : {}",hex(&hk_sim::determinism_suite())),
 Some("replay")=>{let path=args.get(2).ok_or("replay <preuve.json>")?;let text=std::fs::read_to_string(path).map_err(|e|e.to_string())?;let r=hk_net::Replay::verify(&text)?;println!("Rejeu vérifié : {}\n{} ticks, {} cosignataires",hex(&r.hash),r.final_tick,r.players);},
 Some("peer")=>{let id=option("--id").unwrap_or("1".into()).parse::<usize>().map_err(|e|e.to_string())?;let seconds=option("--seconds").unwrap_or("180".into()).parse::<u64>().map_err(|e|e.to_string())?;let autostart=option("--autostart").and_then(|s|s.parse::<usize>().ok());let mode=option("--mode").unwrap_or("skirmish".into());let mut g=hk_core::Game::new("",true);g.save.created=true;g.save.name=format!("PAIR-TEST-{id}");g.save.realm=Realm::from_index(id);g.save.role=Role::from_index(id);if let Some(cell)=option("--cell"){let cell=u64::from_str_radix(cell.trim_start_matches("0x"),16).map_err(|e|e.to_string())?;g.save.world.enter(cell,hk_net::unix_time());}g.screen=7;g.start_network();if let Some(addr)=option("--connect"){g.connect(addr);}if let Some(addr)=option("--relay"){g.node.as_ref().unwrap().reserve(addr);}let start=Instant::now();let mut started=false;let mut last_check=0;let mut printed=0;while start.elapsed().as_secs()<seconds{if let Some(b)=&g.battle{let id=g.online.as_ref().map(|o|o.rollback.local).unwrap_or(0);g.key_input=b.bot_input(id as usize);}g.tick(hk_net::unix_time());if g.addresses.len()>printed{for address in g.addresses.iter().skip(printed){println!("LISTEN {address}");}printed=g.addresses.len();}if let Some(n)=autostart{if !started&&g.peers.len()>=n{g.challenge_mode(if mode=="siege"{2}else if n==1{0}else{1});started=true;println!("CHAMP OUVERT {}",g.peers.len()+1);}}if let Some(o)=&g.online{if g.ticks%150==0{println!("STATUS {}",serde_json::json!({"local":o.rollback.local,"screen":g.screen,"battle_tick":o.rollback.battle.tick,"highest":o.rollback.highest,"ages":o.peer_last_received.iter().map(|t|g.ticks.saturating_sub(*t)).collect::<Vec<_>>(),"finished":o.rollback.battle.finished,"error":o.rollback.error}));}let c=o.rollback.confirmed()/30*30;if c>last_check{if let Some(s)=o.rollback.states.get(&c){println!("CHECK {c} {}",hex(&s.hash()));}last_check=c;}if let Some(e)=&o.rollback.error{return Err(e.clone())}}if g.screen==12{g.finish_online();println!("COMBAT COSIGNÉ");break}std::thread::sleep(Duration::from_millis(33));}if g.online.is_some(){g.finish_online();}if let Some(path)=option("--out"){std::fs::write(path,g.last_proof).map_err(|e|e.to_string())?;}println!("PAIRS {}",g.peers.len());},
 Some("stats")=>{let path=args.get(2).ok_or("stats <export-autorité.json>")?;let v:serde_json::Value=serde_json::from_slice(&std::fs::read(path).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;let seals=v["authority"]["seals"].as_array().ok_or("Export de la Couronne requis")?;let ledger=v["ledger"]["events"].as_array().ok_or("Registre absent")?;let ids:std::collections::BTreeSet<String>=ledger.iter().map(|e|e["body"]["author"].to_string()).collect();println!("Sceaux connus : {}\nÉvénements : {}\nNoms distincts : {}\nAucun indicateur de rétention ou de paiement n'est inventé.",seals.len(),ledger.len(),ids.len());},
 _=>println!("HEXKEEP v0.2\n determinism\n replay preuve.json\n peer --id 1 --seconds 180 [--connect MULTIADDR] [--relay RELAY] [--autostart 9] [--mode siege] [--cell HEX] [--out preuve.json]\n stats export-autorité.json\n Relais : hexkeep-relay 4001")};
    Ok(())
}
