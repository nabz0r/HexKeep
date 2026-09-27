# Livraison 0.4 — Les Chemins de Braise

Version DEV Android du 27 septembre 2026. Le périmètre M0–M7 est conservé ; cette mise à jour porte sur la prise en main, les collisions, les aventures, le butin et leur présentation.

## Ce qui change concrètement

Le joystick conserve sa magnitude, accélère progressivement et freine en deux ticks au maximum. Son origine suit le pouce lors des grands glissements. Mouvement, attaque et visée possèdent des pointeurs indépendants. Un appui maintenu sur ATTAQUE vise un ennemi visible ; un glissement donne la visée manuelle. L’attaque automatique reste une option.

Les déplacements sont balayés par petits pas, les personnages glissent contre les obstacles et les ennemis calculent un chemin. Les positions de départ sont corrigées si nécessaire. Visée et projectiles respectent les obstacles. La caméra suit le veilleur ; brume adoucie, mini-carte, caches et butin rendent la découverte lisible. Un dessin de fond redondant a été supprimé pour réduire le travail de rendu.

Le carnet d’Éline propose feux, chasse et mémoires. Le sac conserve 60 objets, quatre raretés et trois emplacements. Les meilleurs objets apparaissent d’abord ; comparaison, équipement, recyclage et forge sont accessibles depuis le refuge ou le bilan. Les valeurs de vie suivent le Codex scellé. Deux fioles sont disponibles par expédition. Ces bonus ne sont pas injectés dans les combats entre joueurs.

Le GPS reste facultatif et reprend au premier plan après une activation consentie. Passer en exploration simulée l’arrête. Le retour depuis un rejeu de Haute Cour libère correctement les menus d’aventure et ne récompense pas les anciennes éliminations.

## Vérifications exécutées

| Essai | Résultat | Portée |
|---|---|---|
| Moteur Rust | 64 tests distincts réussis : suite workspace de 63 tests, puis noyau de 14 tests comprenant la nouvelle régression de sortie de rejeu | Simulation, géométrie, inventaire, migration, identité, réseau, Couronne et saison |
| Parcours d’aventure | 27/27 victoires, trois caches obtenues dans chaque sortie, inventaire retrouvé après sérialisation | Trois royaumes × trois rôles × trois contrats ; véritables déplacements et commandes, sans téléportation ni dégâts forcés |
| Android téléphone | Suite de six tests verte, puis contrôles ciblés après les derniers changements | Multitouch, attaque, esquive, pouvoir, pause, sauvegarde chiffrée, inventaire, expédition complète, menus avancés, Phare et rendu |
| Android 4:3 | Trois tests verts, puis nouvelle expédition avec la présentation finale du sac | Affichage 1440×1080, commandes adaptées, butin, équipement, réouverture et localisation injectée |
| Mise à jour | Installation signée par-dessus la 0.3 réussie ; même identité publique, même Nom, XP conservés | Profil d’émulateur ; aucune désinstallation requise |
| Déterminisme | 10 000 combats complets, chacun rejoué deux fois, même empreinte sur macOS ARM64, macOS x86_64 et Android ARM64 | Test reproductible, pas preuve exhaustive |
| Deux pairs par relais | 83 checkpoints identiques jusqu’au tick 2490 ; partie finie au tick 2496, deux cosignataires ; rejeu vérifié | Circuits libp2p Relay v2 réels sur le banc local |
| Dix participants | Deux Android + huit processus natifs ; 50 checkpoints communs identiques, jusqu’au tick 1500 | Essai de 65 secondes ; chaque pair calcule son état |
| Paquet | Signature v2, versionCode 4, Android 8+, ARMv7/ARM64/x86_64, release non debuggable | Vérification de signature, contenu et alignement ZIP ; bibliothèques 64 bits alignées à au moins 16 Kio |

Empreinte commune du déterminisme :

```text
f43191762e1faeabc961d020ba34150d1372dbeb1ae5447e1c0def3ebf00780c
```

La preuve publique [cosigned-duel.json](../examples/archive/v04-cosigned-duel.json) se rejoue avec le moteur 0.4. Empreinte du résultat : `9f81eaa66291aa39f0389cc193af914fd92591a1784c71da02ba4e0025a6c352`.

Les mesures de rendu varient avec la charge du GPU logiciel SwiftShader : les parcours tactiles réalisés pendant les essais ont donné environ 25–47 images/s sur le format téléphone et 35–60 sur le format 4:3. Le dernier parcours complet, après suppression du fond redondant, mesure 47,15 images/s pendant 20,21 secondes. Les relevés bruts accompagnent la livraison locale. Il ne s’agit pas de performances de téléphones physiques, ni d’une promesse universelle de 60 images/s. Le banc mixte à dix reste en dessous des 30 ticks/s visés lorsqu’il ralentit, malgré la concordance des états.

## Corrections trouvées pendant les essais

- Le test de réouverture supposait que les XP resteraient strictement identiques ; la veille peut légitimement ajouter un éclat pendant la relance. Il vérifie désormais la conservation de la progression, sans traiter ce gain comme une perte de sauvegarde.
- Le premier paquet d’instrumentation ne contenait pas encore le nouveau parcours tactile complet ; il a été reconstruit, puis le parcours a été exécuté avec succès.
- Les premiers contours de brume et les socles de décor étaient trop géométriques. Le masque est adouci et les ressources de ruines sont cadrées selon leur alpha, sans les présenter comme des cartes d’interface.
- Le retour depuis un rejeu pouvait conserver un combat en lecture et bloquer le nouveau sac. Une régression vérifie sa fermeture, l’absence de récompenses indues et la reprise des aventures.

Aucun échec final des contrôles locaux exécutés n’est laissé sans correction.

## Installation et reproductibilité

Le fichier principal est [HEXKEEP-v0.4.apk](../downloads/HEXKEEP-v0.4.apk). Son empreinte est dans [SHA256SUMS.txt](../downloads/SHA256SUMS.txt). Il emploie le même certificat DEV que les versions précédentes : `497a8facf989aaaf6e18f0d5d7131620b96f9a11b8660d67281477b276a976dc`.

Le dépôt conserve son commit initial et sa licence AGPL-3.0. La notice MIT des versions antérieures et celles des dépendances sont conservées. L’APK, les sources, les ressources, le README illustré, les guides et les tests sont publiés sur `main`. Les clés locales privées, installations d’outils, données d’appareil et sorties temporaires restent exclues du dépôt. La clé opérationnelle de Couronne **DEV**, explicitement partagée pour le laboratoire, reste distincte de la racine privée exclue.

[Construire et jouer](../README.md) · [Protocoles de test](test-protocols.md) · [Manuel M0–M7](gm-manual.md) · [Choix de conception](design-v04.md)

## Limites ouvertes

- **Production :** cette livraison demeure une DEV. Attestation complète, Genèse officielle, distribution Play et paiements réels ne sont pas qualifiés. Le verrou de production reste actif.
- **MMO :** réseau entre pairs jusqu’à dix participants, territoire et fonctions de royaume ; les contrats et le butin 0.4 sont locaux. Aucun monde public permanent avec des milliers de joueurs simultanés n’est revendiqué.
- **Matériel :** aucun téléphone physique n’était connecté. Radio BLE, deux opérateurs mobiles, GPS en promenade réelle, consommation de batterie et biométrie matérielle restent à qualifier.
- **Contenu :** trois contrats, trois familles de marches, trois dispositions de terrain, neuf combinaisons royaume/rôle. La carte garde H3 et les lieux visités ; ce n’est pas une cartographie routière ou un guide d’accès aux lieux réels.
- **Économie :** équipement PvE local, sans marché partagé ni protection de production contre la modification du client. Les combats réseau restent normalisés.
