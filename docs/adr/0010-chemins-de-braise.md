# 0010 — Prise en main, découverte et équipement PvE

## Décision

Préserver la direction artistique peinte de la 0.3, remplacer les fondations de mouvement et ajouter une boucle de progression locale immédiatement lisible. Les commandes tactiles restent distinctes de la simulation entière ; le réseau continue à échanger des commandes et à comparer des états indépendants.

Le joystick garde son amplitude. Accélération, freinage, glissement et navigation ne dépendent pas de la fréquence d’affichage. L’esquive ne traverse plus les obstacles. Le bouton ATTAQUE assiste la sélection d’une cible visible ; le glissement permet une visée manuelle. L’automatisation complète est un réglage facultatif.

Le sac est persistant, borné et réservé aux aventures. Les combattants réseau sont reconstruits sans consulter cet équipement. Les récompenses sont consommées lors de la sortie du combat et non à chaque ouverture du bilan. Un rejeu est fermé sans passer par l’attribution des récompenses d’un combat joué.

## Conséquences

La sensation de mouvement et les chemins changent ; le réseau 0.4 utilise son propre identifiant et ses propres topics. Les sauvegardes de profil migrent avec un équipement de départ. Les anciennes preuves demandent leur moteur d’origine. Les contrats locaux et la forge ne constituent pas une économie de production sécurisée.

Les trois royaumes, neuf combinaisons et fonctions M0–M7 restent accessibles. La publication sur `main` conserve l’historique du dépôt d’accueil et sa licence ; les éléments de validation sont associés à l’APK principale signée.
