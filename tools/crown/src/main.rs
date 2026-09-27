use hk_crown::*;
use hk_proto::{hex, Hash};
use std::{fs, path::Path};
fn write(path: &str, value: &impl serde::Serialize) -> Result<(), String> {
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).map_err(|e| e.to_string())
}
fn read<T: serde::de::DeserializeOwned>(path: &str) -> Result<T, String> {
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
fn private(path: &str, secret: &Hash) -> Result<(), String> {
    use std::io::Write;
    let mut o = fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let mut f = o.open(path).map_err(|e| e.to_string())?;
    f.write_all(&serde_json::to_vec(secret).unwrap())
        .map_err(|e| e.to_string())
}
fn main() {
    if let Err(e) = run() {
        eprintln!("{e}");
        std::process::exit(1)
    }
}
fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let opt = |key: &str, default: &str| {
        args.iter()
            .position(|v| v == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or(default.into())
    };
    let now = opt(
        "--time",
        &std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_string(),
    )
    .parse::<u64>()
    .map_err(|e| e.to_string())?;
    let root_path = opt("--root", "root.json");
    let net = opt("--network", "dev");
    let out = opt("--out", "authority.json");
    match args.get(1).map(String::as_str){
 Some("init")=>{let secret=hk_crypto::new_secret();private(&root_path,&secret)?;let directory=Path::new(&root_path).parent().unwrap_or(Path::new("."));let backup=directory.join("crown-backup.json");let data=serde_json::json!({"mnemonic":hk_crypto::mnemonic(&secret),"shares":hk_crypto::split(&secret).iter().map(|s|hex(s)).collect::<Vec<_>>()});let mut o=fs::OpenOptions::new();o.write(true).create_new(true);#[cfg(unix)]{use std::os::unix::fs::OpenOptionsExt;o.mode(0o600);}use std::io::Write;o.open(&backup).map_err(|e|e.to_string())?.write_all(&serde_json::to_vec_pretty(&data).unwrap()).map_err(|e|e.to_string())?;println!("Racine créée : {}. Sauvegarde privée : {}",hex(&hk_crypto::public(&secret)),backup.display());},
 Some("genesis")=>{let root:Hash=read(&root_path)?;let codex=if args.iter().any(|s|s=="--codex"){read(&opt("--codex",""))?}else{Codex{season_start:now,..Default::default()}};write(&out,&genesis(&root,&net,now,codex))?;},
 Some("certify-throne")=>{let root:Hash=read(&root_path)?;let key:Hash=read(&opt("--public","throne-public.json"))?;let cert=certify(&root,&net,key,now);write(&out,&cert)?;let qr=qrcode::QrCode::new(serde_json::to_vec(&cert).unwrap()).map_err(|e|e.to_string())?;fs::write(format!("{out}.svg"),qr.render::<qrcode::render::svg::Color>().min_dimensions(512,512).build()).map_err(|e|e.to_string())?;},
 Some("revoke-throne")=>{let root:Hash=read(&root_path)?;let key:Hash=read(&opt("--public","throne-public.json"))?;let mut a:Authority=read(&opt("--authority","authority.json"))?;a.propose(&root,EdictKind::RevokeThrone{throne:key},now)?;write(&out,&a)?;},
 Some("codex")=>{let root:Hash=read(&root_path)?;let codex:Codex=read(&opt("--file","codex.json"))?;if !codex.valid(){return Err("Codex invalide".into())}write(&out,&Signed::new(&root,&net,"codex",codex))?;},
 Some("dev-fixture")=>{if net!="dev"{return Err("Fixture uniquement DEV".into())}let root:Hash=read(&root_path)?;let throne=hk_crypto::new_secret();let mut a=Authority::new(genesis(&root,"dev",now,Codex{season_start:now,..Default::default()}))?;a.add_certificate(certify(&root,"dev",hk_crypto::public(&throne),now))?;write(&out,&serde_json::json!({"authority":a,"throne":throne}))?;},
 _=>println!("crown init --root root.json\ncrown genesis --root root.json --network dev --out genesis.json\ncrown certify-throne --root root.json --public throne-public.json --out certificate.json\ncrown revoke-throne --authority authority.json --public throne-public.json --out revoked.json\ncrown codex --file codex.json --out codex-signed.json\nAucun accès réseau. Clés privées : fichiers 0600. JSON canonisé par Borsh pour les signatures.")};
    Ok(())
}
