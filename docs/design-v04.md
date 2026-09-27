# Les Chemins de Braise — intentions et références

## Le problème à résoudre

Le joystick de la 0.3 normalisait tout mouvement non nul à la même vitesse. Le point d’origine pouvait être déplacé au toucher, les ennemis tournaient localement devant les obstacles, le cadrage montrait toute l’arène en permanence et la progression ne proposait pas de vrai sac. La 0.4 corrige ces causes avant d’ajouter de nouveaux objectifs.

Le contrat devient : choisir un but lisible, explorer, combattre, récupérer, comparer et repartir. Les objets donnent des choix et de la continuité. La carte géographique donne des lieux à retrouver. Le multijoueur conserve son périmètre réel et ses valeurs normalisées.

## Héritage étudié

- **World of Warcraft** : objectifs identifiés dans le monde et récompenses explicites. HEXKEEP expose trois contrats, leur état, leur renouveau et leur récompense avant de partir. [Présentation officielle des quêtes mondiales](https://worldofwarcraft.blizzard.com/en-gb/news/23832634/weekly-bonus-event-world-quests).
- **League of Legends** : rôles identifiables, objectifs et progression par objets. HEXKEEP conserve ses trois rôles, montre les intentions ennemies et compare les objets dans le sac. [Guide officiel](https://www.leagueoflegends.com/en-us/how-to-play/).
- **Dark Age of Camelot** : trois royaumes, forteresses et enjeux territoriaux. HEXKEEP garde ses royaumes et les bastions au cœur de la direction MMO. [Présentation officielle du RvR](https://www.darkageofcamelot.com/rvr-server-types/).
- **Quake III Arena** : mouvement réactif et cohérence de la simulation. HEXKEEP emploie sa propre accélération entière, son freinage et ses collisions balayées. [Source de référence publiée par id Software](https://github.com/id-Software/Quake-III-Arena/blob/master/code/game/bg_pmove.c). Aucun code de ce fichier n’a été repris.

Ce sont des choix de conception inspirés par ces références, pas des affirmations d’équivalence avec leurs systèmes ou leur volume de contenu. Aucune marque, musique, carte ou ressource de ces jeux n’est incluse dans HEXKEEP.

## Règles de la version

Simulation entière à 30 Hz, vitesse analogique bornée, accélération de 16 sous-unités par tick et freinage de 24. Les déplacements rapides sont subdivisés en pas d’au plus 24 sous-unités. Rayon de collision : 64 sous-unités, soit un quart de case. Les ennemis suivent un chemin déterministe à quatre voisins puis simplifient les segments visibles. La visée et les projectiles respectent tous les obstacles.

La caméra rapproche le terrain, indépendamment de la taille des commandes. Les zones non visitées sont masquées pendant la sortie ; la mini-carte conserve les objectifs. La silhouette peinte des obstacles est contenue dans un socle qui matérialise leur surface bloquante.

Le butin est local à la DEV et ne constitue pas une économie MMO sécurisée contre la modification du client. Les récompenses sont consommées une fois par sortie. Les bonus de première victoire sont identifiés par cellule, période et contrat. Les combats réseau ne reçoivent aucun bonus de sac. Les anciennes preuves de simulation demandent leur version de moteur ; la preuve d’exemple du dépôt est actualisée pour la 0.4.
