# HEXKEEP v0.3 — Les Braises

Un monde que les joueurs tiennent allumé. Jeu Android natif, simulation Rust entière à 30 Hz, présentation Android native avec illustrations peintes, trois royaumes et trois rôles.

## Installer

Le fichier principal est **`artifacts/HEXKEEP-v0.3.apk`** : DEV release signée, Android 8 ou ultérieur, ARM64 / ARMv7 / x86_64. Ouvrir le fichier sur le téléphone, autoriser ponctuellement l'installation depuis l'application qui l'ouvre, puis installer. La mise à jour de la DEV v0.1/v0.2 conserve le Nom et le monde. Garder ses 24 mots avant toute désinstallation.

Ne pas utiliser `app-prod-release-unsigned.apk` pour installer directement : c'est un artefact destiné à la signature de publication. La variante production reste verrouillée tant que sa chaîne d'attestation et sa Genèse officielles ne sont pas configurées.

## Jouer

1. Entrer dans la nuit : trois scènes de prologue, choix du veilleur et apprentissage jouable. Le Nom est créé et sauvegardé automatiquement ; retrouver ses 24 mots dans **Réglages → Mon Nom & sauvegarde**.
2. Le pouce gauche déplace le veilleur. L’assistance vise et attaque les ombres proches ; glisser à droite reprend la visée manuelle. Les deux boutons déclenchent l’esquive et le pouvoir. Musique, sons, vibrations et assistance se règlent séparément.
3. **Partir en expédition** : rester trois secondes près de chaque balise, repousser les renforts puis dissiper le gardien. Trois lumières permettent de revenir après une chute. Le bilan rejoint la progression du refuge.
4. **Explorer la marche** conserve la sélection de cellules, le GPS facultatif, le déplacement DEV, les bastions et la bannière. **Forteresse & chroniques** ouvre sièges, Courses de Mémoire, saison, boutique, Maisons, Phare et Trône.
5. La boutique est gratuite en DEV, sans débit ; les cosmétiques ne changent pas les statistiques de combat. La piste de saison comporte 40 paliers. Voir [le manuel](docs/gm-manual.md) pour les fonctions M0–M7.

Les personnages, le sol et les décors sont de nouvelles illustrations ; le jeu conserve une vue en deux dimensions. [Direction artistique, fichiers et prompts](docs/art/v03-direction.md).

## Construire

Prérequis : Rust stable, JDK 17, SDK Android 36, NDK `28.2.13676358`, `cargo-ndk`.

```sh
rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android
cargo install cargo-ndk --locked
scripts/build.sh
```

Le script teste le workspace, régénère UniFFI, compile les trois architectures et construit la DEV debug, la DEV release signée et la production unsigned. `JAVA_HOME`, `ANDROID_HOME` et `ANDROID_NDK_HOME` peuvent désigner les installations locales. Le JDK local `.tools/` n'est pas livré dans l'archive.

```sh
adb install -r artifacts/HEXKEEP-v0.3.apk
```

Une signature de publication distincte se configure dans Gradle ou avec `apksigner`. La clé debug de développement ne doit jamais servir à publier la production.

## Réseau local et dix participants

Dans **PAIRS**, ouvrir sa bannière. La découverte mDNS est complétée par **Connexion directe**, qui accepte une multiadresse libp2p. Les deux joueurs doivent être dans la même cellule de test et utiliser cette version. Une adresse est de la forme `/ip4/192.168.1.20/tcp/40001/p2p/IDENTIFIANT`.

```sh
cargo build --release -p hexkeep-sim -p hexkeep-relay -p crown
hexkeep-sim peer --id 0 --seconds 200 --autostart 9
hexkeep-sim peer --id 1 --seconds 200 --connect MULTIADRESSE
```

Lancer neuf autres pairs (ou huit, avec deux Android au total). `--mode siege` ouvre un siège ; `--cell HEX` choisit la cellule. Sans `--autostart`, un pair attend le défi d'un autre. `--out preuve.json` exporte le dossier de combat. Les trois preuves complètes les plus récentes restent aussi dans l'application.

```sh
hexkeep-sim replay preuve.json
hexkeep-sim determinism
python3 scripts/network-smoke.py direct 10 50
python3 scripts/network-smoke.py relay 2 90
python3 scripts/android-ten.py
```

Le dernier script requiert deux émulateurs Pixel 5 déjà installés avec l'APK et son APK d'instrumentation ; il utilise leurs ports 5554/5556. Les checks de hash portent sur des simulations indépendantes, pas une diffusion d'état serveur.

## Relais aveugle

```sh
hexkeep-relay 4001
```

Ouvrir le port TCP/UDP choisi. Le relais affiche sa multiadresse publique avec son Peer ID. Sur le premier téléphone, **Connexion directe → Réserver un relais** ; sa nouvelle adresse contient `/p2p-circuit/p2p/SON_ID`. Le second téléphone s'y connecte. En CLI : `hexkeep-sim peer --relay ADRESSE_RELAIS`. QUIC, TCP/Noise/Yamux, relais v2 et DCUtR sont inclus. Le relais ne possède aucune clé de Nom ni de Couronne et ne stocke aucun monde. Les réservations et circuits sont temporaires et limités.

## Couronne et documentation

La cérémonie DEV et les commandes hors ligne sont décrites dans [le manuel GM](docs/gm-manual.md). La racine privée reste hors dépôt et hors APK. La DEV contient une délégation administrative partagée pour les tests, valable trente jours ; elle est volontairement impropre à une production ouverte.

- [Rapport v0.3](docs/delivery-v03.md) · [Archive v0.2](docs/delivery.md)
- [Protocoles de test](docs/test-protocols.md)
- [Architecture](docs/architecture.md) et [protocole](docs/protocol.md)
- [Économie](docs/economy.md), [vie privée](docs/privacy.md), [menaces](docs/threat-model.md)
- [Bible](docs/bible.md), [héritage de conception](docs/design-heritage.md), [décisions](docs/adr/)
