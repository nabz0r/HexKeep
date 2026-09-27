# Validation de la v0.2

Exécution locale du 27 septembre 2026. Deux AVD Pixel 5 ARM64 sous Android 15 / API 35, macOS ARM64, exécution x86_64 par Rosetta. Aucun téléphone physique, modem 4G ou lecteur biométrique matériel n'a été testé. Les valeurs ci-dessous ne qualifient pas un déploiement de production.

| Critère | Résultat observé | Portée |
|---|---|---|
| A1 Solo hors ligne | Installation, création du Nom, tutoriel tactile, carte, fondation, sauvegarde Keystore chiffrée et réouverture réussis sur les deux AVD en mode avion. Mise à jour depuis la v0.1 également essayée. | Validé sur ce banc Android. |
| A2 Deux appareils | Duel complet entre deux processus via un relais v2 réel, signé 2/2, 596 ticks, rejeu identique. Les deux Android participent au test à dix. | mDNS en moins de 5 s et BLE entre téléphones physiques non qualifiés. |
| A3 Dix joueurs | Deux Android + huit processus, 65 s : 44 checkpoints identiques jusqu’au tick 1320 en 1080p ; 54 jusqu’au tick 1620 en 720p. Aucun état divergent ni arrêt réseau sur ces deux essais finaux. Test de dix processus : 36 checkpoints communs identiques jusqu'au tick 1080. Siège à dix combattants : Porte détruite, ennemi contestant la Cour, victoire à 1800 ticks de capture ; Mémoire lâchée au seuil simulé de 14 km/h. | Siège et Mémoire validés par les tests du moteur ; aucun essai de dix humains dehors. |
| A4 Internet | Transport Relay v2, circuit chiffré et duel cosigné testés ; TCP, QUIC et DCUtR inclus. | Deux opérateurs 4G et leurs latences non testés. |
| A5 Couronne | Cérémonie CLI réalisée ; Genèse, certificats, Merkle et signatures contrôlés. Trône Android, Sceau, Ban après Sceau, import par un autre client, Rollback et saison testés. Haute Cour vérifie la preuve et son rejeu. | Tests locaux ; partage de la délégation DEV explicitement assumé. |
| A6 Parjure | La production est verrouillée. | Non validé : contrôle de l'attestation de production et de la signature de l'APK à terminer. |
| A7 Partition | Deux histoires séparées, temps logique de trente minutes, réunion puis même état canonique et Sceau : test réussi. | Le scénario réel de deux groupes déconnectés pendant trente minutes n'a pas été exécuté. |
| A8 Déterminisme | 10 000 combats complets, chacun rejoué deux fois, empreinte identique en ARM64, x86_64 et dans Android ARM64. Le moteur termine au plus tard après trois minutes de simulation ; cela inclut les parties finies par cette limite. | Test reproductible, pas preuve exhaustive du moteur. |
| A9 Identité | BIP39, toutes les combinaisons de trois parts sur cinq, chiffrement des parts, restauration DEV, certificats Nom/appareil/session. P-256 réellement signée dans Android Keystore et vérifiée par Rust. | Biométrie matérielle et retour de trois parts entre gardiens humains non qualifiés. |
| A10 Performance/Phare | Phare poursuivant ses ticks après mise en arrière-plan, arrêté sans Wi-Fi, vérifié sur AVD. En combat réseau : 28,9–29,3 images/s en 1080p et 53.7–54.6 images/s en 720p. Le Phare au premier plan a rendu 43.8 images/s pendant la dernière vérification, simultanée au test natif sur le second AVD. Le rendu des deux AVD utilise le GPU logiciel SwiftShader ; les performances physiques ne sont pas extrapolées. | Aucune consommation physique mesurée ; aucune garantie de 60 i/s sur milieu de gamme 2021. |

## Suite du moteur

49 tests Rust réussis, dont migration de sauvegarde v0.1, neuf combinaisons, 10 000 combats complets, sessions invalides/expirées, Shamir/chiffrement, Sceaux/Merkle/révocation de Trône, siège, garde anti-Sybil, saison et cosmétiques sans avantage statistique. Le test de réception dans le désordre impose une commande manquante suivie de commandes ultérieures : la prédiction s'arrête à sa fenêtre, puis rattrape exactement la simulation ordonnée lorsque l'entrée arrive. Un autre test détecte un seul pair silencieux même si les autres continuent d'émettre.

Empreinte du déterminisme :

```
dd62be09d7256250da027202439f46ee4080d3a4d0787ffa51c0ab6034f09181
```

Preuve publique fournie, `examples/cosigned-duel.json` : 596 ticks, deux cosignataires ; empreinte :

```
4eaa31bd5bb2e7dfa920eb782062ea8fe12d602376d36b90f97b1e4c214b4b10
```

## Reproduire

```sh
cargo test --workspace --locked --release
cargo test -p hk-sim --release --target x86_64-apple-darwin ten_thousand_replays -- --nocapture
cargo build --release -p hexkeep-sim -p hexkeep-relay -p crown
target/release/hexkeep-sim replay examples/cosigned-duel.json
python3 scripts/network-smoke.py direct 10 50
python3 scripts/network-smoke.py relay 2 90
```

Pour Android, construire `assembleDevDebugAndroidTest`, installer son APK et l'APK signée de l'application avec le même certificat. Les tests sont lancés avec `am instrument` et AndroidJUnitRunner. `GameplayTest` couvre le solo et le déterminisme, `ExpansionTest` la campagne/Keystore/boutique/Trône, `PhareTest` la veille. Le Phare exige Wi-Fi, secteur et autorisation GPS précise, y compris sur l'émulateur.

`python3 scripts/android-ten.py` requiert les AVD `emulator-5554` et `emulator-5556`, profils créés et le même Codex initial que les huit pairs CLI. Ne pas modifier/sceller un Codex dans un seul profil juste avant ce test : le refus d'un autre Codex est intentionnel. Le script compare des états calculés séparément. Sa fermeture volontaire des processus met fin à l'essai ; elle ne constitue pas une victoire cosignée.

Les coordonnées tactiles du test solo correspondent au Pixel 5 de test en paysage 2340×1080. Le moteur vérifie aussi le rendu des écrans de campagne aux largeurs internes 400, 520, 584 et 640.

## Échecs rencontrés et corrigés

- APK unsigned non installable ; DEV limitée aux architectures 64 bits et Android 9 : livraison principale signée, trois ABI, Android 8 minimum.
- Résolution DNS lisant un fichier absent sous Android : utilisation du résolveur de la plateforme pour les noms, transport natif initialisé sans ce fichier.
- Adresse de circuit de relais contenant deux fois le Peer ID : normalisation corrigée, puis duel Relay v2 terminé et vérifié.
- Nom de Maison DEV trop long : corrigé, puis parcours Android réussi.
- Paquet reçu hors ordre entraînant un dépassement du rollback, puis des forfaits en cascade : progression limitée aux entrées contiguës et retransmission bornée des entrées manquantes.
- Vérifications de certificats répétées à haute fréquence : cache borné de chaînes validées, signature et expiration encore vérifiées pour chaque paquet ; tests de falsification après cache réussis.
- Réservation arrivant avant Identify au relais : annonce immédiate de ses adresses d'écoute, nouveau duel cosigné réussi.
- Phare refusé alors que le banc était encore en mode avion : arrêt conforme à sa règle ; essai réussi après réactivation du Wi-Fi.

Les 49 tests du moteur, les deux essais mixtes à dix, le duel final par relais et les parcours Android finaux sont verts. Aucun échec automatisé final n'est laissé sans correction. Les critères non qualifiés A4, A6, A7 réel, A9 matériel/social complet et A10 ne sont pas déclarés réussis. A2 et A3 restent partiels sur matériel physique.

## Contrôles de paquet

Signature APK v2 vérifiée ; cible API 36, minimum API 26, trois bibliothèques natives ARMv7/ARM64/x86_64. Alignement ZIP vérifié avec des pages de 16 Kio. La DEV release n'a pas l'indicateur `debuggable`, bien que sa clé soit volontairement une clé de développement. Voir les sorties originales `v02-signature.txt` et `v02-package.txt` dans la validation livrée.

Le banc à dix enregistre l'heure monotone de chaque checkpoint et le nombre de trames rendues. L'objectif d'acceptation du test de 65 secondes est au moins 1 200 ticks et 40 checkpoints communs, sans erreur réseau. Les scripts effacent uniquement leurs anciennes sorties de test pour qu'un résultat précédent ne puisse pas masquer un nouvel échec.

## Mesures brutes du banc

- 1080p, 5554: 28.86 images/s ; 22.50 ticks/s mesurés ; 924 trames >33ms ; 66.29 secondes.
- 1080p, 5556: 29.33 images/s ; 22.01 ticks/s mesurés ; 902 trames >33ms ; 65.80 secondes.
- 720p, 5554: 53.67 images/s ; 26.61 ticks/s mesurés ; 118 trames >33ms ; 65.76 secondes.
- 720p, 5556: 54.59 images/s ; 26.73 ticks/s mesurés ; 97 trames >33ms ; 65.87 secondes.

La fréquence logique cible reste 30 Hz ; la progression réellement observée ci-dessus est inférieure quand le banc ralentit. Le critère complet de fluidité A3/A10 sur téléphones reste donc à qualifier, même si les états réseau sont cohérents.

## Reprise v0.3

`GameplayTest` utilise les coordonnées de la présentation native et vérifie les gestes à deux doigts, la pause, le retour depuis le bilan, le changement de format, le stockage et la relance. `IdentityMigrationTest` enregistre uniquement l’identité publique avant remplacement de l’APK et la compare ensuite. Les commandes du profil de test sont indépendantes de l’identité réelle d’un joueur.

Avant `PhareTest`, connecter le Wi-Fi de l’émulateur et conserver l’alimentation. Le test accorde les permissions de localisation et de notification à son seul paquet de test. Le service doit rester actif pendant la mise en arrière-plan et s’arrêter sur demande.

Pour le test à dix, utiliser des profils DEV de test sur la même marche, avec le Codex initial. `NetworkHarness` interroge directement le moteur synchronisé depuis le fil de test ; attendre que la boucle de dessin devienne inactive fausse un échantillonnage temps réel. Le script exige au moins 40 points de contrôle communs et aucune divergence.

`cargo run --release -p hk-core --example expedition` joue neuf expéditions par déplacement réel et visée assistée, sans téléporter le joueur ni forcer les dégâts. `cargo run --release -p hk-apu --example score -- extrait.wav` reproduit l’extrait sonore livré.

## Reprise v0.4

Le [rapport courant](delivery-v04.md) remplace les résultats historiques ci-dessus pour la version 0.4. Le protocole de simulation a changé ; les empreintes et preuves antérieures ne sont pas comparables avec celles de la 0.4.

```sh
cargo test --workspace --locked --release
cargo run --release -p hk-core --example adventure
cargo test -p hk-sim --release --target x86_64-apple-darwin ten_thousand_replays -- --nocapture
```

`AdventureTest` réalise une sortie complète par événements tactiles : deux doigts maintiennent déplacement et attaque ; le test parcourt trois caches, les objectifs et le gardien, puis équipe le meilleur butin via les boutons et vérifie la réouverture. Aucun adversaire n’est supprimé artificiellement. La mesure de rendu couvre ce déplacement effectif, avec un seuil de régression de 20 images/s sur le GPU logiciel du banc.

`GeolocationTest` injecte des callbacks de position Android pour vérifier l’acceptation d’une marche, le rejet d’une mesure imprécise et d’un saut trop rapide, la découverte H3 et l’arrêt du GPS en voyage simulé. Il ne remplace pas une promenade physique. `IdentityMigrationTest` compare la clé publique et les progrès avant et après installation de l’APK signée. Les tests ignorent les gains passifs légitimes d’XP lors d’une relance.

Pour le 4:3, configurer l’émulateur de test avec `adb shell wm size 1080x1440`, exécuter le parcours, puis `adb shell wm size reset`. Les coordonnées sont converties par le même facteur d’échelle et le même décalage vertical que la vue Android.

Après les essais de Trône et de GPS, le banc à dix doit employer des profils d’émulateurs dédiés remis au Codex initial et à la même cellule. Archiver les captures avant de réinitialiser ces seuls profils de test. Ne jamais effacer les données d’un téléphone utilisateur pour ce banc.
