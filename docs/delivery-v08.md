# Livraison 0.8 — Le Serment des Lanternes

Version Android **8 / 0.8.0-lanternes**, Rust **0.8.0**, Android 8 minimum, API cible 36. Cette livraison refait la direction artistique des Confins et modernise aussi les sprites des aventures précédentes.

## Ce qui change

- Trois héros et sept créatures peints, dont les trois gardiens propres à leurs biomes ; 120 poses d’animation. Les états de déplacement, d’attaque, d’esquive et d’impact pilotent les images.
- 72 objets illustrés dans les six catégories. Comparaison, rareté, tri, glisser-déposer et couches de matière sur le personnage sont conservés.
- Trois sols et 36 éléments de décor peints : arbres, rochers, ruines, balises, campements et reliques. Décors triés avec les personnages, transparence devant le joueur, exploration avec brume interpolée et cadrage rapproché.
- Nouveau refuge, atlas sur parchemin, personnages voyageurs, contrôles illustrés et panneaux sombres bordés d’or. La minimap se replie près d’un boss pour dégager la vue.
- Cache partagé limité à 88 Mio de références résidentes, images chargées à la demande, anciennes planches retirées de l’APK mais conservées comme sources dans Git.
- Les trois régions, douze missions, cinématiques, postures, combos, anciennes aventures et sauvegardes de la 0.7 restent disponibles. Aucun multijoueur de production n’est ajouté à l’édition Play solo.

## Validation

Les preuves sont dans [validation/v08](validation/v08/).

| Contrôle | Résultat |
|---|---|
| Workspace Rust | 88 tests réussis |
| Android et bibliothèques natives | Compilation DEV, PROD et Play ; ARM64, ARMv7 et x86_64 ; Lint sans erreur, 15 avertissements |
| Prévisualisation Android 16 | 12 tests d’instrumentation sur la version finale : installation, cycle de vie, anciennes aventures, campagne tactile, équipement, formats et ressources graphiques |
| Bundle Android 16 | Installation des APK fractionnés produits depuis l’AAB, signature locale de test ; 12 tests d’instrumentation ; système à pages de 16 384 octets |
| Android 8 | Première passe complète de 12 tests ; après les derniers ajustements, 8 tests de campagne, rendu, anciennes aventures et cycle de vie réexécutés avec succès |
| Mise à jour DEV | Installation 0.7 puis 0.8, sans désinstaller : identité publique, nom, XP, objets, caractéristiques, six équipements et poussières conservés |
| Graphismes | 26 ressources vérifiées, cellules non vides, transparence des sprites, six couches visibles, vues des trois héros et des trois régions |
| Tailles d’écran | Cibles tactiles vérifiées en formats allongé, 16:9 et 4:3 ; scènes de rendu supplémentaires à 960, 1200 et 1440 pixels logiques |
| Paquets Play | Version 8, cible 36, seule permission `VIBRATE`, ressources 0.8 incluses, trois ABI, alignement natif 16 Kio ; AAB sans service réseau ni facturation |

La scène tactile de campagne sur l’APK final Android 16 a mesuré **35,1 images/s en moyenne** et environ **127 Mio de tas natif**. Les mesures sont des parcours instrumentés sur émulateur SwiftShader, avec menus et chargements ; elles ne certifient ni un débit constant ni les performances thermiques d’un téléphone. Le cache de 88 Mio borne les images conservées par le renderer, pas la mémoire totale du processus.

Les anciens tests tactiles attendent désormais que le refuge ait effectivement présenté ses contrôles avant de lire les coordonnées et d’envoyer les gestes. Les transitions attendent deux images rendues, et chaque événement injecté doit être accepté. Cette synchronisation corrige les deux démarrages trop précoces détectés sur le runner distant ; les parcours complets de butin et d’exploration ont été rejoués avec succès (`android36-sync-*`).

Les captures `v08-renderer-*` sont des états de test rendus par les classes du jeu. Les captures sans ce préfixe et celles de `store/fr-FR/screenshots/` proviennent de parties et de gestes exécutés sur Android. La présentation des gardiens en fixture ne prétend pas être une victoire jouée.

## Installer et retrouver les sources

- [APK Play de prévisualisation](../downloads/HEXKEEP-v0.8-PLAY-preview.apk).
- [APK DEV de mise à jour](../downloads/HEXKEEP-v0.8-DEV.apk).
- [Empreintes SHA-256](../downloads/SHA256SUMS.txt).
- `scripts/build.sh` produit aussi `artifacts/HEXKEEP-v0.8-PLAY.aab`. Le bundle est disponible dans les artefacts de la CI.
- [Architecture](architecture-v08.md), [direction artistique](art/v08-direction.md), [prompts ImageGen](art/v08-prompts.json), [guide de jeu](player-guide-v07.md).

Les équipements appliquent six zones de matière au sprite animé ; les 72 illustrations du sac ne sont pas 72 silhouettes d’armes entièrement animées et interchangeables. Cette distinction est détaillée dans la direction artistique.

La release est livrée dans le dépôt GitHub. L’AAB de publication reste non signé avec une clé officielle ; aucun envoi Play Console n’a été effectué. La signature du propriétaire, les déclarations de boutique et les essais sur téléphones physiques restent les étapes du [dossier Google Play](play-release.md).
