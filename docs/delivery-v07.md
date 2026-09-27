# Livraison 0.7 — Les Échos des Confins

Version Android **7 / 0.7.0-confins**, Rust **0.7.0**, Android 8 minimum, API cible 36. La livraison comprend les sources, les APK Play et DEV, le candidat AAB, la documentation et les preuves de validation. La distribution officielle Google Play n’a pas été soumise.

## Contenu livré

- Trois nouvelles régions de 48 × 32 cases : Sylve des Échos, Dunes de Verre, Couronne Boréale. Soit 4 608 cases supplémentaires, neuf balises, douze mémoires, trois voyageurs et trois gardiens.
- Atlas global interactif avec exploration conservée et régions débloquées par les gardiens.
- Trois missions principales et neuf secondaires ; acceptation, suivi et récompense persistante. Dix cinématiques avec texte progressif, caméra, pause et passage.
- Combat local à huit états, combo de quatre coups, hitstop, secousses légères, esquive, éclat et fioles. Trois postures pour le veilleur et des postures ennemies fonctionnelles.
- Cinq nouveaux archétypes d’IA, dont le gardien à trois phases : poursuite, recul à distance, garde, essaim mobile, attaques radiales/charge/invocations.
- Six emplacements, 72 modèles d’objets, silhouette composée de couches. Grille tactile avec comparaison, tri, rangement manuel, glisser-déposer et infobulle au survol.
- Accueil, journal, chroniques, bestiaire, minimap, transitions et respect des zones sûres. Rendu procédural des nouveaux contenus ; ressources historiques conservées dans les anciens modes.
- Reprise du combat et des cinématiques après fermeture ; aucune récompense de mission ou de bilan attribuée deux fois. Les cinq aventures et les fonctions DEV restent disponibles.

## Validation

Les journaux sont regroupés dans [validation/v07](validation/v07/). Ils distinguent tests du moteur, tests d’application installée et inspection des paquets.

| Vérification | Résultat |
|---|---|
| Suite Rust complète | 88 tests réussis, dont 37 dans le cœur du jeu ; déterminisme réseau historique inclus |
| Campagne des Confins | Trois régions et douze missions terminées par déplacement, combat et interactions normaux ; 6 724 ticks, sans téléportation, modification de PV ni attribution forcée |
| Anciennes aventures | 135 / 135 parcours hors ligne réussis : royaumes × rôles × contrats × difficultés ; trois caches et reprise de sauvegarde vérifiées |
| Endurance des nouvelles IA | Dix minutes simulées en fixture synthétique d’immunité : limites des monstres, projectiles, effets et taille de sauvegarde contrôlées |
| Paquets | APK Play signée de prévisualisation, APK DEV signée, AAB reproductible, trois ABI ; lint Android : 0 erreur, 15 avertissements documentés |
| Contrôle Play | `game.hexkeep`, version 7, cible 36, seule permission `VIBRATE`, aucune facturation ni service réseau dans le manifeste ; bibliothèques 64 bits et APK alignés 16 Ko |
| Android 16 / pages 16 Ko | Neuf parcours d’instrumentation sur les APK fractionnés issus de l’AAB |
| Android 8 / API 26 | Neuf parcours d’instrumentation sur les APK fractionnés issus de l’AAB |
| Gestes de la 0.7 | Atlas verrouillé, acceptation, chroniques, reprise de cinématique, tri, drag & drop correct/incorrect, survol, combo, posture, butin gagné au combat et équipement sans soin gratuit |
| Migration APK DEV | Installation 0.6 puis mise à jour 0.7 : identité, nom, XP, objets, valeurs, trois anciens emplacements et poussières conservés ; trois nouveaux emplacements ajoutés |
| Formats | Vérification des cibles dans la zone sûre en formats allongé, 16:9 et 4:3 ; captures réelles et retour Android |

Les performances enregistrées dans `touch-performance-*.json` proviennent d’émulateurs avec rendu logiciel. Elles ne qualifient pas un téléphone commercial. Le jeu borne ses listes transitoires et réutilise son terrain ; le rapport distingue la taille de tas observée d’une mesure de consommation ou de stabilité thermique.

## Installer et construire

- [APK Play de prévisualisation](../downloads/HEXKEEP-v0.7-PLAY-preview.apk), sans compte ni connexion.
- [APK DEV de mise à jour](../downloads/HEXKEEP-v0.7-DEV.apk), avec le certificat des éditions DEV précédentes.
- Empreintes : [SHA256SUMS](../downloads/SHA256SUMS.txt).
- `scripts/build.sh` reconstruit les variantes et l’AAB `artifacts/HEXKEEP-v0.7-PLAY.aab`. Les artefacts de la CI contiennent également le bundle.
- `scripts/test-play-device.sh` lance les parcours séparément pour éviter qu’un geste abandonné ne contamine le suivant.

L’[architecture](architecture-v07.md) donne les chemins des systèmes et leurs invariants ; le [guide de jeu](player-guide-v07.md) explique les postures et la progression. Les [références de conception](design-v07.md) documentent les choix.

## Périmètre restant pour Google Play

L’AAB livré est non signé pour l’envoi officiel. La clé du propriétaire, Play App Signing, les déclarations, le contact public, le rapport de pré-lancement et les essais sur téléphones restent à traiter dans le [dossier Google Play](play-release.md). L’application est française ; elle ne propose pas encore de navigation TalkBack complète, de synchronisation des sauvegardes ou de MMO de production.

Les nouvelles illustrations sont volontairement procédurales. Les combats historiques conservent leurs sprites peints ; les six nouvelles couches sont visibles dans la campagne des Confins et dans l’aperçu du sac. Un ancien sac plein peut conserver temporairement trois pièces supplémentaires lors de la migration, afin de ne supprimer aucun objet existant ; il faut recycler de la place pour recommencer à stocker du butin.
