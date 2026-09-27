# Historique des versions

## 0.8.0 — Le Serment des Lanternes · 27 septembre 2026

- Refonte artistique peinte : trois héros, quatre familles de monstres et trois boss uniques, avec 120 poses d’animation au total. Les anciennes aventures utilisent aussi les nouvelles planches.
- 72 illustrations d’équipement, six catégories, cadres de rareté, comparaison et six couches de matière visibles sur le héros.
- Trois sols et 36 décors peints : Sylve, Dunes, Couronne. Ruines, végétation, balises, campements et reliques ; profondeur, transparence des obstacles devant le joueur, brume douce et cadrage rapproché.
- Nouveau refuge illustré, atlas sur parchemin, voyageurs peints, huit médaillons d’aptitudes et interface sombre à bordures dorées.
- Cache d’images partagé et borné, chargement paresseux, retrait des anciennes planches inutilisées du paquet Android. Les sources historiques restent dans le dépôt.
- Tests graphiques de toutes les planches et régions, animations, six couches et trois formats de canevas, en complément des parcours tactiles de campagne et des anciennes aventures.
- Version Android 8 ; règles de jeu, progression, sauvegardes et édition Play solo hors ligne conservées. Voir [la livraison](docs/delivery-v08.md) et [l’architecture](docs/architecture-v08.md).

## 0.7.0 — Les Échos des Confins · 27 septembre 2026

- Campagne originale en trois régions de 48 × 32 cases : forêt inondée, désert cristallin et toundra céleste. Atlas interactif, voyages, progression et déverrouillage des territoires.
- Douze missions, dont trois principales et neuf secondaires : balises et gardiens, mémoires, voyageurs à secourir, chasses. Acceptation, suivi et récompenses persistantes attribuées une seule fois.
- Dix cinématiques de campagne : introductions, révélation, arrivées des gardiens et conclusions. Caméras interpolées, texte progressif, pause, passage et reprise après fermeture.
- Machine à états du combat : déplacement, préparation, frappe, récupération, esquive, impact et mort. Combo trois coups rapides → finissante lourde, brise-garde, fenêtre d’enchaînement, commandes tamponnées pendant le hitstop.
- Postures Équilibre, Assaut et Rempart : vitesse, puissance, résistance et silhouette. Postures ennemies liées à leur comportement.
- Cinq archétypes supplémentaires : Ravageur agressif, Aiguilleur à distance, Bastion gardé, Essaim mobile et Archonte à trois phases. Trois gardiens nommés, projectiles radiaux, charges et invocations.
- Six emplacements : arme, plastron, amulette, casque, gants et bottes. Catalogue de 72 modèles ; migration additive conservant les objets, valeurs et identités historiques.
- Nouvel inventaire tactile : grille paginée, glisser-déposer vers les emplacements, déplacement des cases, tri, comparaison chiffrée, infobulle au survol, forge et recyclage au refuge.
- Superposition de six couches procédurales sur le personnage de campagne et son aperçu. Biomes, silhouettes, icônes, effets et atlas dessinés par le code ; aucun nouvel asset externe nécessaire.
- Accueil, atlas, journal, chroniques et bestiaire cohérents ; transitions, minimap, zones sûres, mode contraste et réduction des secousses.
- Sauvegarde complète du combat, des phases, de l’exploration, du butin et des cinématiques. Menus et interruptions suspendent la campagne ; reprendre reste une action volontaire.
- Anciennes aventures et fonctions DEV conservées. Aucun changement des règles du combat réseau ; édition Play toujours solo hors ligne, sans publicité ni achats intégrés.
- Version Android 7, API cible 36, trois ABI. Tests Rust, parcours tactiles Android et contrôle APK/AAB ; détails et limites de validation dans `docs/delivery-v07.md`.

## 0.6.0 — Les Lanternes du Refuge · 27 septembre 2026

- Édition Play solo hors ligne jouable, AAB, signature d’envoi configurable et contrôles des artefacts. Le réseau de production reste verrouillé ; la DEV conserve ses fonctions.
- Sauvegarde et reprise volontaire des aventures après fermeture, sans nouvelle attribution du même bilan. Protection d’un fichier illisible et nouvelle tentative après échec d’écriture.
- Refuge peint original, visuel Google Play, icône adaptative et monochrome ; accueil recentré sur les aventures et la collection.
- Pause, aide, réglages en aventure, confirmation avant abandon, contraste et réduction des animations d’ambiance, respect des zones sûres d’écran.
- Focus audio, interruption sur retrait des écouteurs et retour Android 16. Protection de fenêtre modifiée seulement lors d’un changement réel.
- Édition Play sans autorisation Internet/GPS/Bluetooth, sans SDK Billing et sans service de veille. Confidentialité et licences consultables hors ligne.
- Dossier de lancement, fiche française, captures et banc de tests API 36. Publication Google Play non effectuée ; conditions restantes dans `docs/play-release.md`.

## 0.5.0 — L’Éveil des Veilleurs

- Sept atlas originaux et 84 poses : marche de face/de dos, gauche/droite, attaque, esquive, impact, chute et effets des pouvoirs. Animation liée à la distance et aux actions réelles.
- Sac et statistiques accessibles depuis la barre de vie ou **Sac · stats** en combat. Pause solo, équipement sans soin gratuit, reprise du même affrontement.
- Objectif, consigne et direction toujours visibles ; fioles, interactions, dangers au sol et entrée du gardien expliqués à l’écran.
- Sept comportements ennemis : rôdeur, tisseur, sentinelle, oracle invocateur, traqueur, pilleur et gardien dont les secousses accélèrent à mi-vie.
- Deux nouveaux contrats : veillée de 90 secondes et exploration de cinq curiosités. Cinq coffres par sortie, source, cloche, autel facultatif et Minuit le chat.
- 36 modèles d’objets, quatre serments, bonus de deux pièces, collection, bestiaire et neuf mémoires originales conservées dans le carnet.
- Trois difficultés : Découverte, Périlleux après trois victoires, Éclipse après neuf. Butin épique garanti sur le gardien en Éclipse.
- Migration additive 0.4, objets et identité conservés, même signature DEV, version Android 5. Navigation corrigée au contact des angles de mur ; protocole de combat 0.5 séparé pour préserver le déterminisme.

## 0.4.0 — Les Chemins de Braise

- Commandes analogiques progressives, freinage court, joystick flottant qui suit le pouce ; multitouch conservé pendant esquive et pouvoir.
- Collisions balayées avec glissement, positions de départ valides, navigation des ennemis autour des obstacles, visée assistée avec visibilité et anticipation du mouvement.
- Caméra rapprochée, mini-carte, exploration de la brume, caches et ramassage de braises ; secteurs visités conservés sur la carte géographique.
- Carnet d’Éline : trois contrats, récompense de première victoire par marche et période de trente minutes, reprises possibles.
- Sac de 60 objets, quatre raretés, arme/manteau/relique, comparaison, statistiques PvE, recyclage, forge et deux soins par expédition.
- Refuge et accueil multijoueur remaniés, intentions d’attaque plus visibles, bilan du butin.
- Migration additive des sauvegardes 0.2/0.3 ; identité et progression conservées. Protocole de simulation 0.4 distinct.
- Publication du projet, APK signée, captures et documentation sur `main`.

## 0.3.0 — Les Braises

Direction artistique peinte, rendu Android natif, prologue en trois scènes, héros, musique originale, trois balises et gardien, menus rénovés. Voir [le rapport](docs/delivery-v03.md).

## 0.2.0 — Jalons M0 à M7 en DEV

Identité, veille géographique, combats et rejeux, réseau à dix, relais, campagne, Maisons, Phare, Trône et chaîne d’autorité DEV. APK principale signée, Android 8+, trois architectures. Voir [le rapport](docs/delivery.md).
