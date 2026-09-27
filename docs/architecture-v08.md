# Architecture graphique 0.8

Le moteur Rust, les règles de combat, les missions et la sauvegarde restent ceux des Confins. La 0.8 remplace leur présentation Android. Aucun serveur ni accès réseau n’est ajouté à l’édition Play.

| Responsabilité | Chemin depuis la racine |
|---|---|
| Images sources et planches d’animation | `android/app/src/main/assets/art/v08/` |
| Chargement, cadres alpha et cache LRU partagé de 88 Mio | `android/app/src/main/java/game/hexkeep/art/PaintedArt.kt` |
| Personnage, poses et six couches de matière | `android/app/src/main/java/game/hexkeep/frontier/EquipmentLayers.kt` |
| Bestiaire animé et télégraphes | `android/app/src/main/java/game/hexkeep/frontier/EntityRenderer.kt` |
| Sols, placement des décors et brume | `android/app/src/main/java/game/hexkeep/frontier/SceneryRenderer.kt` |
| Caméra, profondeur, effets, HUD et minimap | `android/app/src/main/java/game/hexkeep/frontier/FrontierRenderer.kt` |
| Atlas interactif illustré | `android/app/src/main/java/game/hexkeep/frontier/AtlasScreen.kt` |
| Sac, comparaison, tri et glisser-déposer | `android/app/src/main/java/game/hexkeep/frontier/InventoryScreen.kt` |
| Peau d’interface et zones tactiles | `android/app/src/main/java/game/hexkeep/frontier/UiKit.kt` |
| Anciennes aventures et événements d’animation | `android/app/src/main/java/game/hexkeep/ActorAnimator.kt` |
| Cinématiques et mouvements de caméra | `android/app/src/main/java/game/hexkeep/frontier/CutsceneDirector.kt` |
| Machine à états et combat autoritaires | `crates/hk-core/src/frontier/combat.rs` et `entities.rs` |
| Tests des ressources et scènes graphiques | `android/app/src/androidTest/java/game/hexkeep/ArtDirectionTest.kt` |

## Cycle de vie

`NightSurface` possède un seul `PaintedArt`, partagé avec les anciennes aventures et `FrontierUi`. Le décodage est paresseux. Les textures opaques utilisent RGB565 et les sprites ARGB8888. Le cache borne ses références résidentes à 88 Mio et expose des métriques ; une texture évincée n’est pas recyclée pendant qu’un Canvas peut encore l’utiliser. La fermeture de la surface libère les ressources encore détenues.

Les cellules sont cadrées une fois au chargement, en parcourant une ligne de pixels à la fois. Aucun tableau de pixels de la taille complète de la planche n’est conservé. Le cadrage de chaque pose conserve le rapport largeur/hauteur. Les anciens compteurs d’animation restent liés aux événements de simulation.

`SceneryRenderer` prépare les décors lors du changement de région. `FrontierRenderer` fusionne leur ordre vertical avec celui des monstres et du héros. Les objets lointains sont écartés du rendu. Les objectifs et les interactions utilisent les positions autoritaires du contenu JSON. La brume est une image de 48 × 32 pixels, mise à jour uniquement si l’exploration change.

## Reproduction

`scripts/build.sh` teste le workspace Rust, produit les bibliothèques Android pour trois ABI, compile les éditions et exécute Lint. `scripts/test-play-device.sh` lance les parcours Android et les tests graphiques. `scripts/verify-play.py` inspecte les APK/AAB, leurs permissions, les ABI, l’alignement 16 Kio et les ressources 0.8.
