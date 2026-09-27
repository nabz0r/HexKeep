# Livraison 0.5 — L’Éveil des Veilleurs

Version DEV Android du 27 septembre 2026. Les fonctions M0–M7 sont conservées. La mise à jour concerne le mouvement visible, la lecture du combat, le sac en aventure, les rencontres et la progression.

## Le changement en jeu

Les illustrations fixes ont été remplacées par sept planches originales totalisant 84 poses. Le veilleur montre son dos lorsqu’il s’éloigne, change d’appui en marchant, se retourne, frappe et esquive. Les ennemis suivent leurs propres déplacements et attaques. Les impacts, pouvoirs et chutes ont un retour visuel. Les ruines sont ordonnées avec les acteurs pour une profondeur cohérente.

La barre de vie et **Sac · stats** ouvrent les caractéristiques et le matériel pendant l’aventure. Le solo se suspend, le butin déjà trouvé est consultable et l’équipement peut être changé sans restaurer gratuitement la vie. Revenir retrouve le même combat. Les instructions, le prochain repère, les fioles et les interactions sont visibles dans le HUD.

Cinq contrats, sept familles d’ennemis, cinq coffres par sortie, une source, une cloche, un autel et Minuit enrichissent les rencontres. Le carnet conserve 36 modèles d’objets et neuf mémoires à lire. Quatre serments donnent un bonus de deux pièces. Les difficultés Périlleux et Éclipse s’ouvrent après trois et neuf victoires.

## Vérifications

| Contrôle | Résultat observé |
|---|---|
| Moteur | **72 tests réussis** : combat, navigation, sauvegarde, équipement, secrets, réseau et jalons M0–M7 |
| Aventures | **135/135 victoires**, trois caches récupérées dans chaque parcours et inventaire conservé après rechargement ; commandes réelles et équipement du butin trouvé |
| Téléphone émulé 2340×1080 | Six tests d’acceptation réussis : animations et curiosités, expédition complète, gestes et cycle de vie, fonctions avancées, Phare et GPS injecté |
| Format 4:3 1280×960 | Animations, sac en combat, secrets, expédition, équipement et réouverture réussis |
| Mise à jour 0.4 → 0.5 | Installation par-dessus, identité publique, Nom, XP, identifiants et valeurs des objets, équipement et braises conservés |
| Animation observée | Huit poses de marche du joueur, changements de face/dos et gauche/droite, attaques, esquive et attaques ennemies ; captures et compteurs exportés |
| Déterminisme | **10 000 combats complets**, rejoués deux fois chacun ; même empreinte macOS ARM64, macOS x86_64 et Android ARM64 |
| Deux pairs par relais | 35 checkpoints communs identiques jusqu’au tick 1050 ; fin au tick 1071, deux cosignataires, rejeu vérifié |
| Dix participants | Deux Android + huit pairs natifs, **54 checkpoints identiques jusqu’au tick 1620**, aucune divergence ; deux AVD en 1560×720 |
| APK | Signature v2 vérifiée, même certificat DEV, versionCode 5, Android 8+, ARM64/ARMv7/x86_64 ; alignement ZIP et 11 bibliothèques natives contrôlés |

Les deux expéditions tactiles mesurées donnent **43,40 images/s sur 20,51 s** en format téléphone et **59,81 images/s sur 19,83 s** en 4:3. Le banc utilise SwiftShader, sur émulateurs Android 15/API 35. Ces valeurs décrivent ces essais, pas des téléphones commerciaux. Le banc réseau à dix atteint environ 29–30 images/s et 25 ticks/s en 720p. Un premier essai avec les deux rendus plus grands n’atteignait que 1170 ticks en 65 s, sous le seuil de 1200 du banc, sans divergence détectée. La qualification du réseau à dix à 30 ticks/s constants en haute résolution reste ouverte ; le passage à 720p valide la concordance des simulations sans prétendre résoudre cette limite matérielle.

Empreinte commune des 10 000 combats : `015c7661915282832b72f2943daa4b53888f18f01fc1b0034a2a7a79ac0ef997`.

La preuve publique [cosigned-duel.json](../examples/cosigned-duel.json) se rejoue avec le moteur 0.5 : `b0d83f00665de17da9c45d8669368cd36573f3d4bba4409b83a61f0738dda7d9`. La preuve 0.4 est conservée dans `examples/archive/`.

Les journaux, captures, compteurs et une vidéo de présentation accompagnent la livraison locale. La reconstruction indépendante Linux, macOS et Android est consultable dans [GitHub Actions](https://github.com/nabz0r/HexKeep/actions).

## Corrections issues des essais

- Une position valide contre un angle arrondi pouvait être enfermée par le test rectangulaire conservateur utilisé pour le chemin. La navigation emploie désormais la collision circulaire réelle dans ce cas. Une régression couvre le contact de mur et l’angle.
- Le pilote tactile utilisait une marge rectangulaire excessive et une annulation incohérente d’un geste à deux doigts. Il a été aligné sur la géométrie réelle et relâche maintenant chaque pointeur, sans déplacer artificiellement le personnage.
- Le libellé du pouvoir consultait un champ de présentation absent. Il lit maintenant le rôle du combattant : Surcharge, Protection ou Entrave.
- Les notifications du sac et du carnet pouvaient masquer la pagination ; elles occupent désormais une ligne sous le titre. Les annonces de butin en combat passent sous la zone centrale pour garder le héros visible.
- Les objets anciens recevaient des métadonnées incomplètes de collection. La migration ajoute ces métadonnées sans modifier leurs identifiants ni caractéristiques.
- Le bilan recompterait les caches si leur attribution immédiate n’était pas mémorisée. Les caches et curiosités sont marquées, avec des tests d’attribution unique et de conservation au retour.

## Périmètre

Le paquet est une DEV signée. Les aventures, objets et secrets restent locaux ; le réseau entre pairs et les fonctions de royaume ne constituent pas un serveur MMO public permanent. Les combats réseau utilisent le Codex commun, sans bonus d’équipement PvE.

La correction des trajectoires impose le protocole **0.5** sur tous les participants. Les profils migrent depuis la 0.4 ; les preuves historiques doivent être rejouées avec leur moteur d’origine. Aucun téléphone physique n’était disponible : compatibilité radio réelle, consommation, GPS en promenade et performances d’appareils commerciaux restent à qualifier. Les essais SwiftShader ne sont pas une promesse de fréquence d’affichage universelle.

[APK](../downloads/HEXKEEP-v0.5.apk) · [Guide](player-guide.md) · [Conception](design-v05.md) · [Art](art/v05-atlases.md) · [Protocoles de test](test-protocols.md)


## Paquet livré

`HEXKEEP-v0.5.apk` : **43 585 035 octets** (environ 43,6 Mo), application `game.hexkeep.dev`, version `0.5.0-eveil`. SHA-256 :

```
19017b134b4dda4ce954204df2cb1acabfa243850547881dbd323f26bef48c1b
```

Certificat DEV inchangé : `497a8facf989aaaf6e18f0d5d7131620b96f9a11b8660d67281477b276a976dc`. L’APK principale est signée et installable ; la production unsigned n’est pas proposée comme téléchargement joueur. Mettre à jour sans désinstaller conserve la sauvegarde. Les clés privées de construction restent exclues du dépôt.
