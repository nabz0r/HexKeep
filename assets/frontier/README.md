# Les Échos des Confins

`campaign.json` est le catalogue immuable de la 0.7 : trois régions, douze missions, dix cinématiques et cinq archétypes de monstres. `chronicles.txt` rassemble les textes consultables dans les archives du jeu. Le moteur inclut ces fichiers à la compilation ; aucun téléchargement n’est nécessaire.

Les positions sont exprimées en cases de 256 unités. Une région mesure 48 × 32 cases. Les obstacles sont des rectangles `[x, y, largeur, hauteur]` ; les objectifs sont des coordonnées `[x, y]`. Les chronologies sont mesurées en ticks de 1/30 seconde. Un identifiant de mission ou de scène déjà publié doit rester stable pour préserver les sauvegardes.

Les nouvelles ressources visuelles sont produites par le code Android dans `frontier/` : géométrie, palettes, couches, contours et transparences. Aucun sprite, logo, texture ou texte provenant des jeux de référence n’est incorporé. Les textes et le code de cette campagne sont des contributions originales au projet et suivent sa licence.

Vérification : `cargo test -p hk-core --release frontier::tests`. Le test de contenu vérifie que chaque objectif est joignable depuis le point de départ ; la traversée complète exerce le combat et les missions par les commandes normales.
