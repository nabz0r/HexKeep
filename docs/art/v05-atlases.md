# Art et animation — L’Éveil des Veilleurs

## Production

Mode utilisé : **ImageGen intégré**, génération à partir du brief et de la référence graphique `art/characters.png`, puis une édition de transparence pour l’atlas des curiosités. Aucun fichier d’un jeu commercial n’est incorporé. Les images livrées sont les sorties du générateur ; le cadrage de chaque cellule est calculé en mémoire par Android à partir de son canal alpha.

Le jeu reste une représentation 2D peinte, avec de nouvelles poses dessinées pour les actions. Il ne s’agit pas d’animations 3D. Les effets de frappe, de pouvoir, de choc, de chute et les traînées sont composés par le rendu.

## Brief commun des planches

Un atlas original pour HEXKEEP, fantasy sombre peinte, cohérent avec les personnages existants. Fond réellement transparent, grille de quatre colonnes et trois lignes égales, silhouette entière dans chaque cellule, même échelle et pieds ancrés de façon cohérente. Pas de texte, décor ni cadre. Première ligne : quatre phases de course vers le bas à droite. Deuxième ligne : quatre phases de course vers le haut à droite, dos visible. Dernière ligne : frappe de face, frappe de dos, esquive de face, esquive de dos. Des poses et appuis distincts, lisibles à petite taille. Les orientations à gauche sont obtenues par miroir au rendu.

| Fichier livré dans `android/app/src/main/assets/art/v05/` | Sujet | Dimensions |
|---|---|---|
| `aurelon.png` | Veilleur d’or, lame et lanterne | 1254 × 1254 |
| `skarn.png` | Archer d’argent et de givre | 1254 × 1254 |
| `vylde.png` | Mystique vert, bâton et sève | 1254 × 1254 |
| `wolf.png` | Loup spectral de cendre | 1254 × 1254 |
| `wraith.png` | Spectre drapé et magie violette | 1254 × 1254 |
| `golem.png` | Gardien de pierre massif | 1254 × 1254 |
| `pilleur.png` | Petit pilleur masqué, bec de bronze, capuche bleu pétrole, sac de braises | 1254 × 1254 |
| `curiosities.png` | Huit objets de découverte | 1536 × 1024 |

Les sept personnages fournissent **84 poses sources**. L’oracle et le traqueur réemploient les silhouettes du spectre et du loup avec leur couleur et comportement propres.

## Brief des curiosités

Atlas transparent de quatre colonnes et deux lignes, objets peints en vue de dessus inclinée : coffre fermé, coffre ouvert, source de lune, arche à cloche, autel rouge, chat noir avec lanterne, lanterne avec livres, tas de reliques et braises. Même direction artistique, pas de texte, chaque objet isolé. Une édition finale a retiré le fond résiduel en conservant les objets et leur disposition. Les six premiers éléments sont employés comme objets interactifs ; les deux derniers restent des ressources disponibles.

## Règles de mouvement

La phase de marche dépend de la distance réellement parcourue ; rester bloqué contre un mur ne produit pas une course sur place. La direction est celle du déplacement, ou de la visée pendant une attaque. Les hausses des compteurs de recharge déclenchent les poses de frappe, d’esquive et de pouvoir. La perte de vie déclenche l’impact. Le système s’applique aux joueurs et aux ennemis ; les décors passent devant ou derrière selon leur profondeur. Le mode de mouvements réduits retire les traînées et les étincelles secondaires.

Les dimensions ci-dessus sont celles des fichiers réellement livrés, pas la taille demandée au générateur. Les sept feuilles possèdent un canal alpha vérifié, comme les curiosités. Les essais visuels et tactiles figurent dans [le rapport 0.5](../delivery-v05.md).
