# Architecture livrée

Kotlin possède le cycle Android, les permissions, le GPS, le Keystore AES-GCM, l’entrée multitouch, AudioTrack et GLSurfaceView. UniFFI transporte commandes et images vers Rust. `hk-ppu` fabrique un tampon RGBA de hauteur 240 ; GL le présente avec filtrage NEAREST et facteur entier, centré. La simulation fonctionne à 30 Hz ; le rendu vise 60 Hz. Le retard accumulé est borné pour éviter une explosion de rattrapage après une pause.

`hk-proto` définit les entiers fixes, entrées et royaumes. `hk-sim` possède le combat, les projectiles, armure, pickups et IA. `hk-world` convertit les positions locales en H3 et applique veille/Noir/fondations. `hk-crypto` fournit Ed25519, BIP39 et Shamir. `hk-ledger` signe et ordonne le DAG de développement. `hk-net` utilise libp2p TCP, Noise, Yamux, mDNS, gossipsub, identify et ping ; il conserve les états de rollback. `hk-core` relie les modules et dessine tous les écrans. `hk-apu` synthétise quatre voix. Aucun SDK d’analytics, de publicité, d’authentification distante ou de crash reporting.

Le thread GL cadence le jeu ; le thread AudioTrack tire le PCM ; un runtime Tokio dédié possède les sockets. Les accès au cœur sont sérialisés. Les clés privées ne quittent pas la sauvegarde chiffrée et la mémoire du processus. La couche transport utilise aussi une identité éphémère libp2p. Tous les joueurs exécutent le combat ; le premier pair ne dispose d’aucune autorité persistante. Le choix initial des participants par le challenger reste un protocole dev simplifié.

La conversion GPS→H3 et le rendu sont hors simulation. Le combat n’utilise aucun flottant, et les états sont sérialisés Borsh avant BLAKE3. Le temps local n’est utilisé que pour la veille provisoire dev. Aucun état n’est présenté comme scellé.
