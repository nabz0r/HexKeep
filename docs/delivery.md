# Livraison du 27 septembre 2026

**Une alpha dev installable et jouable est livrée. Le cahier des charges complet M0–M7 n’est pas terminé.** Le mode de production est volontairement verrouillé ; l’APK à installer est `app-dev-debug.apk`.

## Fichiers

- `artifacts/app-dev-debug.apk` : APK signé avec la clé Android de développement, ARM64 et x86_64, Android 9 minimum, cible 36.
- `artifacts/app-prod-release-unsigned.apk` : variante unsigned, écran de production fermé. Aucune Genèse officielle ou attestation de production n’est simulée.
- `artifacts/hexkeep-source.zip` : dépôt source, tests, assets originaux, documentation et exemple de preuve cosignée.
- `artifacts/SHA256SUMS` : empreintes des livrables.

## Jalons

| Jalon | Livraison réelle |
|---|---|
| M0 | Workspace Rust, projet Android Kotlin/Gradle, UniFFI, écran titre, synthèse audio et CI écrite. Build local réussi ; la CI distante n’a pas été exécutée. |
| M1 | Combat jouable, neuf combinaisons royaume/rôle, armure, dash, compétences, pickups, IA et tutoriel. Les détails de physique et l’équilibrage ne reproduisent pas encore intégralement le Codex cible. |
| M2 | Carte H3, veille au premier plan, Noir, fondation dev, chronique, journal signé et sauvegarde chiffrée. Boucle solo fonctionnelle. Garde collective/bastions de production non finalisés. |
| M3 | BIP39, Nom Ed25519, récupération de mots, chiffrement Keystore, ouverture par biométrie/code si configuré. Shamir testé comme bibliothèque. Hiérarchie appareil/session et attestation hors ligne manquantes. |
| M4 | libp2p local chiffré, découverte mDNS, connexion IP, présence de bannière, petits journaux, combats déterministes et preuves cosignées. mDNS non fiable sur tout le banc d’essai ; BLE absent. |
| M5 | Combat réseau à dix vérifié avec deux AVD + huit processus. Pas de transport Internet mobile, siège, Course de Mémoire, Phare ou reprise après partition de trente minutes. |
| M6 | Non livré : CLI crown, Trône, Haute Cour, Genèse, Sceaux, Édits, Merkle, révocations et attestations prod. |
| M7 | Lore original, rangs et 30 Proclamations documentés. Aucun achat, boutique, Maison, saison jouable, Nuit Longue ou Chronique scellée. Un thème principal et des effets simples seulement ; orchestration musicale complète restante. Traduction anglaise partielle. |

## Résultats vérifiés

- **27 tests Rust verts**, dont primitives cryptographiques, Shamir, fold, H3, veille/Noir, simulation, rollback et transport TCP/Noise réel.
- Deux tests Android réussis sur chacun des deux AVD : cycle solo hors ligne et déterminisme natif.
- Test mixte : **deux Android + huit pairs**, 33 empreintes communes identiques jusqu’au tick 990. Test à dix processus : 31 empreintes communes identiques jusqu’au tick 930.
- Duel terminé au tick **1334**, résultats **cosignés 2/2**, même hash reproduit séparément à partir de chacune des deux preuves.
- Déterminisme : **10 000 scénarios de 180 ticks rejoués deux fois**, même empreinte sur ARM64, x86_64 et Android ARM64. La durée limitée est explicite ; il ne s’agit pas de 10 000 parties terminées.
- APK installé et relancé, signature v2 vérifiée et alignement ZIP pour pages de 16 Kio vérifié.

Aucun test automatisé final n’est rouge. Le test de découverte seule à dix processus a échoué ; le repli par connexion directe a réussi et cette limite est conservée dans le rapport. Les critères non exécutés ou incomplets ne sont pas comptés comme réussis. Les détails et les protocoles restants figurent dans `test-protocols.md`.

## Écarts principaux et justification

Le réseau est exclusivement dev. Il utilise des clés de Nom directement, et des messages signés individuellement, sans certificats de session ni chaîne d’entrées conforme au protocole final. Le snapshot chiffré remplace provisoirement le stockage append-only compacté. Les fondations sont celles du terrain de développement ; aucun placement réel sûr selon OSM n’est revendiqué. Le calcul de garde et l’évolution génétique existent comme primitives testées mais ne gouvernent pas encore toute la boucle collective.

Le combat est une implémentation d’alpha : dash amorti, trajectoires et IA simplifiées, pas de charge Skarn qui renverse au contact, pas d’interpolation visuelle complète, arène à motifs bornés. Les rencontres dev sont limitées à trois minutes ; les duels peuvent se terminer avant par trois rounds. Les PR de réseau ne sont pas attribués ; les victoires solo sont honorifiques et ne changent aucune statistique. Aucun achat n’existe.

Ces choix rendent le jeu testé concret et installable tout en conservant l’isolation de production. Ils ne sont pas une déclaration d’équivalence au MMO final.

## Risques ouverts

- **Critique avant production :** attestation, certificats, autorité de Couronne, finalité et validation des événements collectifs manquantes. Garder prod fermé.
- **Élevé :** sécurité anti-Sybil/DoS non auditée, Internet mobile non pris en charge, limites de découverte LAN, absence des systèmes de siège/Mémoires/Phare et de reprise de partition.
- **Moyen :** équilibrage, accessibilité complète, anglais complet, consommation et performance sur téléphones réels non validés. Un duel interrompu ne possède pas de résultat cosigné complet.

Aucun serveur d’autorité, compte externe, paiement, publication Play ou clé de Couronne réelle n’a été créé. Les fichiers synchronisés du projet sous `sources/` n’ont pas été modifiés.
