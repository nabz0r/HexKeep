# 0.5 — L’Éveil des Veilleurs

## Intention

La 0.4 permettait de gagner et de conserver du butin, mais ses personnages restaient des illustrations immobiles. La 0.5 donne à chaque déplacement et à chaque rencontre un retour visuel immédiatement compréhensible, puis donner une raison concrète de repartir.

## Contrat de jeu

- Des planches originales fournissent les poses de marche de face et de dos, de frappe et d’esquive. Les poses suivent la distance réellement parcourue et les actions réellement déclenchées. Les ennemis utilisent le même système.
- Le portrait de vie et le bouton Sac ouvrent les statistiques et l’équipement depuis une aventure. La partie solo est suspendue pendant la consultation ; revenir restitue exactement le combat en cours. Les commandes tactiles sont libérées en ouvrant un panneau.
- Les objets obtenus aux caches deviennent visibles dans le sac pendant la sortie. Changer d’équipement ne soigne pas gratuitement. La forge reste un service du refuge.
- Chaque objectif affiche une instruction, sa progression et un repère de direction. Les pouvoirs indiquent leur effet et leur délai de recharge.
- Les rencontres mêlent poursuivants, tireurs, sentinelles, invocateurs, élites et gardiens. Leur silhouette, leur intention et leur préparation d’attaque permettent de choisir comment réagir.
- Les sorties proposent des caches supplémentaires, des curiosités narratives, des défis facultatifs et des secrets persistants. Le carnet conserve la collection et annonce la prochaine récompense.
- La difficulté se débloque avec les victoires ; elle augmente les risques et le butin. La progression demeure locale en PvE, indépendante du Codex utilisé entre joueurs.

## Validation

Parcours tactiles avec rotation et changements de poses ; sac ouvert en plein combat, équipement et reprise sans perte d’état ; collecte et secrets ; migration de la 0.4 ; simulations des contrats ; signature et installation de l’APK ; reconstruction publique sur GitHub. Les résultats mesurés et les limites sont consignés dans le rapport de livraison.


Le parcours complet sur les trois difficultés a révélé un blocage de la navigation lorsqu’un déplacement finissait dans la marge d’un angle de mur. La correction compare ces cas à la collision circulaire réelle. Le protocole réseau passe à 0.5 pour garder tous les pairs sur les mêmes règles. Voir [le rapport](delivery-v05.md).
