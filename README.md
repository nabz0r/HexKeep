# HEXKEEP — La Première Nuit

Alpha Android native, réseau **dev**, Rust + Kotlin. Le jeu solo fonctionne sans compte, sans Internet et sans Google Play Services. Le réseau local permet de relier des appareils qui exécutent cette même version. **Cette livraison n’est pas le MMO de production complet du prompt maître.** Voir [le rapport de livraison](docs/delivery.md).

## Installer et jouer

1. Installer `artifacts/app-dev-debug.apk` sur Android 9 ou supérieur, ARM64 ou x86_64. Autoriser l’installation depuis le gestionnaire de fichiers si Android le demande.
2. Ouvrir HEXKEEP en paysage. Accepter les règles, choisir un royaume et un rôle, noter les 24 mots hors du téléphone puis vérifier trois mots.
3. Stick gauche : déplacement. Stick droit : glisser depuis son centre pour viser et maintenir le tir. A : dash. B : compétence. PAUSE permet de reprendre ou de revenir sur la carte.
4. La carte démarre dans une zone **de développement à Luxembourg**. Le GPS est facultatif. Pour le mode simulé, toucher un hexagone puis MARCHER (DEV).
5. Fonder un bastion dans la cellule initiale éclairée. Les cellules voisines sortent du Noir après dix minutes de veille continue. Les cellules sans veille s’éteignent après trente minutes.
6. Le menu permet de changer le nom d’affichage, le son, les vibrations et la palette. Le rôle se change depuis la carte, sans progression de puissance.

L’identité et le monde sont chiffrés localement dans le Keystore Android. Sur un appareil verrouillé, une biométrie ou le code de l’appareil ouvre la session. Sans verrouillage, le réseau dev accepte une clé Keystore non liée à l’authentification. Réinstaller après désinstallation efface la partie locale ; les 24 mots restaurent **l’identité**, pas les événements dont aucune copie ne subsiste.

## Jouer entre appareils

Utiliser le même Wi-Fi, ouvrir **PAIRS**, puis **LEVER BANNIÈRE / CONNECTER** sur chaque appareil. La découverte mDNS doit rendre les autres joueurs visibles. Les joueurs doivent être dans la même cellule de développement. DÉFIER ouvre un duel ; OUVRIR UN CHAMP inclut jusqu’à neuf autres pairs présents. Les boutons affichent les pairs réels, jamais des bots présentés comme des joueurs.

Si le routeur bloque le multicast, saisir l’adresse IP et le port affichés sur l’autre téléphone. Exemple : `192.168.1.20:43210`. Les connexions utilisent TCP/Noise avec identités et messages signés. Les matchs sont de développement, sans PR et sans finalité de Couronne. Passer l’application en arrière-plan coupe le réseau ; un adversaire absent interrompt le combat après cinq secondes.

Les preuves cosignées s’exportent après le combat depuis PAIRS → EXPORT REJEU. Un combat interrompu produit une preuve incomplète explicitement signalée par le simulateur.

## Construire

Prérequis : Rust stable, JDK 17, Android SDK (plateforme 36, build-tools 36.0.0, NDK 28.2.13676358), `cargo-ndk`, cibles `aarch64-linux-android` et `x86_64-linux-android`. `ANDROID_HOME` et `JAVA_HOME` peuvent être définis dans l’environnement.

```sh
rustup target add aarch64-linux-android x86_64-linux-android
cargo install cargo-ndk --locked
./scripts/build.sh
```

Le script teste Rust, génère les bindings Kotlin UniFFI, compile les bibliothèques ARM64/x86_64 et les deux APK. Le script accepte macOS et Linux ; la CI fournie utilise macOS. Les dépendances sont épinglées dans `Cargo.lock` et le wrapper Gradle est inclus.

## Simulateur et dix pairs

```sh
cargo build -p hexkeep-sim --release
./target/release/hexkeep-sim determinism
./target/release/hexkeep-sim replay hexkeep-rejeu.json
./target/release/hexkeep-sim peer --id 1 --seconds 200 --autostart 9 --out preuve.json
```

Dans neuf autres terminaux, lancer `peer --id 2` à `peer --id 10` sur le même réseau. Chaque processus constitue un pair réel, avec ses propres clés, son transport chiffré et sa simulation. `--connect /ip4/127.0.0.1/tcp/PORT` permet une connexion directe. Pour deux téléphones et huit bots, connecter les téléphones dans PAIRS et lancer huit processus `peer`; un téléphone ouvre ensuite le Champ. Les pairs CLI jouent avec le comportement de test des Sans-Témoin. Cela ne constitue pas une mesure sur dix téléphones physiques.

## Production et signature

`app-prod-release-unsigned.apk` est volontairement **verrouillé**. Il ne peut pas rejoindre un réseau prod incomplet. Signer ce fichier ne déverrouille pas le mode production : il faut d’abord terminer l’attestation, la Genèse officielle, la chaîne de Sceaux et les autres jalons manquants.

Pour signer une archive Android quand la production aura été validée : générer une clé de signature privée avec `keytool`, conserver sa sauvegarde hors du dépôt, puis utiliser `zipalign` et `apksigner sign --ks /chemin/cle-privee.jks --out app-prod-release.apk app-prod-release-unsigned.apk`. Ne jamais publier la clé ni utiliser la clé debug comme identité officielle.

## Ce qui n’est pas disponible

Aucun relais Internet, DCUtR, mode Phare, Couronne/Trône, boutique, Maison ni Course de Mémoire opérationnels. Il n’y a donc pas de commande de lancement de relais ou de cérémonie GM prête à exécuter. Les interfaces manquantes ne sont pas remplacées par de faux services. Les primitives Shamir, évolution et garde sont testées comme bibliothèques, avec intégrations restantes décrites dans le rapport.
