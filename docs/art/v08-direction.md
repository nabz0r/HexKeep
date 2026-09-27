# 0.8 — Le Serment des Lanternes

La direction revient à la fantasy peinte des sprites 0.5–0.6. Les formes géométriques qui représentaient les personnages, monstres, objets et obstacles des Confins en 0.7 sont remplacées par des illustrations originales. Le contraste ambre / indigo relie tout le jeu ; les matières distinguent les régions : végétation humide et bronze, grès et prismes, neige et argent.

## Livrables artistiques

26 fichiers PNG dans `android/app/src/main/assets/art/v08/` :

- Trois héros, douze poses chacun : Aurelon, Skarn, Vylde.
- Quatre familles ennemies et trois gardiens, douze poses chacun : ravageur lupin, archer spectral, bastion de pierre, essaim ailé ; Cerf sans Aube, Matriarche du Prisme, Archonte des Derniers Feux.
- Six planches de douze équipements : armes, plastrons, amulettes, casques, gants, bottes. Le catalogue natif détermine l’illustration ; la rareté reste un cadre indépendant.
- Trois sols peints et trois planches de douze éléments de décor : végétation, rochers, ruines, campement, balises, coffres et reliques.
- Refuge, atlas du monde, huit médaillons d’aptitudes et de navigation, trois portraits et trois sprites de voyageurs.

Les illustrations ont été produites avec **OpenAI ImageGen**, en génération ou édition guidée par références. Les prompts exacts, références et tentatives retenues sont consignés dans [v08-prompts.json](v08-prompts.json). Les deux premières versions du décor forestier ont été écartées car leur fond restait visible ; la version finale reprend la transparence de la planche boréale. Aucune image tierce sous licence n’a été importée. Les anciens originaux restent dans Git ; le paquet Android exclut les planches remplacées.

## Intégration

Le rendu Android découpe les cellules des planches selon leurs canaux alpha. Les animations suivent les états du moteur. Le personnage conserve six couches de matière : copies du sprite peint limitées aux zones de casque, torse, arme, gants, bottes et amulette. Changer d’objet modifie ces matières ; le sac affiche les 72 illustrations. Ce système ne prétend pas fournir 72 silhouettes d’armes interchangeables animées sur chaque héros.

Le cadrage des Confins passe à 48 pixels logiques par case et un héros d’environ 82 pixels de haut. Les décors et acteurs sont triés par profondeur. Un obstacle au premier plan devient translucide lorsqu’il masque le joueur. La brume d’exploration est interpolée depuis une petite texture ; les surfaces peintes restent continues.

Les fenêtres sombres et les bordures dorées reprennent les matériaux des personnages. Les contrôles gardent leurs zones tactiles et les écrans restent dans les marges sûres Android. Les animations d’ambiance respectent le réglage de réduction des mouvements.

## Contrôle

Les captures `v08-renderer-*` sont des scènes de validation du véritable moteur de rendu alimenté par des états de test, pas des captures d’une partie jouée. Les autres captures `v08-*` viennent des parcours tactiles sur Android. Les tests couvrent la transparence des planches, les 120 poses d’acteurs, les six couches d’équipement, trois largeurs de canevas, les trois biomes et la limite du cache d’images.
