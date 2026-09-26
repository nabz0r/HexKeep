# Protocoles d’acceptation

Exécutés le 27 septembre 2026, Android 15 / API 35 sur deux AVD ARM64 Pixel 5 ; APK cible API 36. Les AVD ont été créés pour ce projet. Aucun téléphone physique ou opérateur mobile n’a été utilisé.

| Critère | État et preuve |
|---|---|
| A1 Solo hors ligne | Parcours Android en mode avion : installation, consentement, royaume/rôle, mots et vérification, entrée au tutoriel, déplacement tactile, sortie vers carte, fondation, chiffrement et restauration. Test Rust supplémentaire de victoire au tutoriel avec des entrées normales. Réouverture après arrêt forcé vérifiée séparément. |
| A2 Deux appareils | Transport et combat testés ; duel complet entre deux processus indépendants, résultat signé 2/2 et rejeu identique. Deux Android participent aussi au test à dix. mDNS intermittent dans le banc d’essai ; connexion IP fonctionnelle. BLE et preuve de co-présence absents : A2 intégral non validé. |
| A3 Dix joueurs | Deux émulateurs Android + huit processus libp2p : 33 points de contrôle communs, jusqu’au tick 990, tous identiques. Autre test à dix processus : 31 points identiques jusqu’au tick 930. Ni siège ni Course de Mémoire : A3 intégral non validé. |
| A4 Internet/4G | Non exécuté ; DCUtR, QUIC et relais non implémentés. |
| A5 Couronne | Non implémenté. Aucun Sceau produit. |
| A6 Parjure | Non implémenté ; prod entièrement verrouillé. |
| A7 Partition 30 min | Fusion déterministe unitaire testée ; scénario réseau complet de partition non exécuté. |
| A8 Déterminisme | 10 000 scénarios aléatoires de 180 ticks, rejoués deux fois, identiques sur macOS ARM64, exécution x86_64 et Android ARM64. Cela ne constitue pas 10 000 duels complets ni une couverture exhaustive du moteur. |
| A9 Identité/SSO | BIP39 et toutes les combinaisons 3-sur-5 testées dans Rust ; Keystore + sauvegarde chiffrée testés sur Android. Biométrie sur matériel réel, attestation et SSO du Trône non validés. |
| A10 Performance | Rendu et transport observés sur AVD ; aucune mesure de milieu de gamme 2021 ou de consommation Phare. Non validé. |

## Reproduire les tests locaux

`cargo test --workspace --locked --release -- --nocapture` : 27 tests réussis au moment de la livraison. Le test de transport ouvre des sockets temporaires de développement.

`cargo test -p hk-sim --release --target x86_64-apple-darwin -- --nocapture` : exécuté via la compatibilité x86_64 de macOS. Empreinte attendue : `54f7ff9666521f8b0d41b95742763a1abc94268013332efbe700bfb6289c15cc`.

`cd android && ./gradlew connectedDevDebugAndroidTest` : parcours Android et déterminisme natif. Le test UI utilise les coordonnées du Pixel 5 de test, 2340×1080 en paysage ; adapter le générateur de gestes sur un autre format. `NetworkHarness` est ignoré sans argument `peerAddress`.

Pour le test réseau Android, lancer huit `hexkeep-sim peer` dans la même Marche, le premier avec `--autostart 9`. Lancer NetworkHarness sur deux AVD en lui passant `/ip4/10.0.2.2/tcp/<port-hôte>`. Comparer les empreintes de `network-checks.jsonl` aux lignes CHECK des huit pairs. Après fermeture volontaire des AVD, les autres pairs interrompent la partie : les messages de forfait à ce moment sont attendus.

`hexkeep-sim replay examples/cosigned-duel.json` : doit produire le hash `f3c658281962f9eafa04ffcf2e85ddeae54458d4f31ec62441e9018ce31e0eef`, tick 1334, signatures 2/2, sans finalité de Couronne. Les participants et la cellule de cet exemple sont des données de test.

## Vérifications manuelles restantes

Installer sur un appareil physique, vérifier biométrie et code, refus des permissions, GPS extérieur, multigestes simultanés, audio et retour haptique, réception après perte Wi-Fi, lecteurs d’écran et tailles atypiques. Tester les quotas mémoire et batterie avant tout pilote public. Exécuter les scénarios A4–A10 manquants après leur implémentation. Ne pas transformer ces protocoles écrits en résultats de tests prétendument exécutés.
