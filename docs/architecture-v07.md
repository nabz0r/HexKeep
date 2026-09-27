# Architecture des Confins — 0.7

La boucle `Game::tick` conserve son pas de 30 Hz. `NightSurface` lui transmet les commandes et reçoit une présentation JSON via `Engine` (UniFFI). Le rendu Android peut rafraîchir indépendamment la scène. Les nouvelles règles PvE sont dans `hk-core/frontier` ; le moteur réseau `hk-sim` et ses formats de preuve restent inchangés.

| Domaine | Fichiers | Responsabilité |
|---|---|---|
| Contenu | `assets/frontier/campaign.json`, `chronicles.txt` | Régions, palettes, obstacles, objectifs, 12 missions, 10 scènes et bestiaire |
| Données typées | `crates/hk-core/src/frontier/data.rs` | Chargement unique et immuable avec `OnceLock` |
| Entités et états | `frontier/entities.rs` | Joueur, monstres, projectiles, effets, états et postures sérialisables |
| Combat | `frontier/combat.rs` | Machine à états, combo, hitstop, esquive, IA, phases et durées de vie |
| Monde | `frontier/world.rs` | Collisions entières, visibilité, exploration et champ de navigation partagé |
| Missions | `frontier/missions.rs` | Acceptation, compteurs plafonnés, suivi et état de récompense |
| Cinématiques | `frontier/cinematics.rs` | Chronologie en ticks, texte, caméras et reprise exacte |
| Commandes | `frontier/actions.rs` | Voyages verrouillés, menus, interactions et transactions de récompense |
| Intégration | `frontier/mod.rs` | `Campaign`, `Run`, sauvegarde et présentation publique |
| Équipement partagé | `crates/hk-core/src/adventure.rs` | Six emplacements, migration, statistiques, collection, forge et tri |

Les chemins `frontier/` du tableau sont relatifs à `crates/hk-core/src/`.

## UI Android

Les nouveaux composants sont dans `android/app/src/main/java/game/hexkeep/frontier/` :

- `FrontierUi.kt` : navigation, propriété de chaque doigt, transitions et écrans d’accueil/pause/bilan.
- `UiKit.kt` : palette, typographie, boutons et coordonnées d’interaction utilisées par les tests.
- `AtlasScreen.kt`, `QuestJournalScreen.kt`, `ChronicleScreen.kt` : atlas, missions, lore et bestiaire.
- `InventoryScreen.kt` : grille, pagination, tri, comparaison, survol et glisser-déposer. Une dépose d’équipement vérifie le type d’emplacement ; la règle finale est appliquée dans Rust.
- `FrontierRenderer.kt` : terrain procédural, caméra, brouillard, objets, interface de combat et minimap.
- `EntityRenderer.kt`, `EquipmentLayers.kt` : cinq silhouettes ennemies et six couches d’équipement du veilleur.
- `CutsceneDirector.kt` : interpolation de caméra, bandes cinématiques et révélation du texte depuis la chronologie sauvegardée.
- `CombatFeel.kt` : secousse brève et bornée, appliquée au monde ; les commandes restent fixes. Le réglage de réduction du mouvement supprime la secousse.

`NightSurface` reste le point d’entrée Android et héberge le rendu des anciens modes. Il applique les insets Android avant la conversion des coordonnées, neutralise les commandes lors des transitions et délègue les nouveaux écrans. Le sac précédent a été retiré au profit du composant commun.

## Sauvegarde et transactions

`Save.frontier` possède son propre numéro de schéma. Les anciennes sauvegardes reçoivent une campagne vide. Les identifiants de slot 0/1/2 restent arme/plastron/amulette ; 3/4/5 ajoutent casque/gants/bottes. Les anciennes pièces et leurs valeurs restent intactes. Les trois pièces initiales supplémentaires sont ajoutées une seule fois.

Le `Run` garde positions, PV, postures, FSM, combos, cooldowns, ennemis, phases, projectiles, exploration, découvertes, cinématique et attribution du bilan. Les doigts et requêtes transitoires ne sont pas sérialisés. Après lancement, Continuer ouvre une pause ; Reprendre rétablit ensuite le combat ou la cinématique.

Un objectif ramassé est enregistré avant le retour de la commande. Une mission marquée `claimed` ne peut pas payer deux fois ; un run marqué `rewarded` ne peut pas réattribuer son gardien. Quitter garde exploration, mémoires, quêtes et butin, mais recommence les balises et les ennemis lors du prochain voyage. Une défaite conserve le butin déjà reçu.

Un checkpoint incompatible est signalé comme erreur de stockage et n’écrase jamais le coffre chiffré d’origine. Sa présentation utilise un état neutre afin de ne pas provoquer un crash à l’ouverture.

## Bornes et vérification

Une région comporte 1 536 cases ; un champ de distances est partagé par les ennemis. Le run borne les monstres à 48, les projectiles à 128 et les effets à 64, avec expiration des projectiles, impacts et cadavres. Le rendu conserve un seul bitmap de terrain de 1 536 × 1 024 pixels (6 Mio), recyclé au changement de biome et à la fermeture. Les récompenses récentes sont bornées à 16.

Les tests de `frontier/tests.rs` couvrent la connectivité de tous les objectifs, la campagne complète par commandes normales, les phases, postures, migrations, reprises, répétitions de récompenses, le rejeu d’un checkpoint et une charge synthétique d’IA de dix minutes. `FrontierTest.kt` vérifie l’intégration par gestes réellement injectés dans l’application installée.
