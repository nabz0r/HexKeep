# Direction artistique — Les Braises, v0.3

Quatre illustrations originales générées avec l’outil intégré **imagegen**, puis intégrées telles quelles au dépôt. Aucun appel API externe ni image empruntée à un jeu commercial. Le découpage des atlas se fait en mémoire par le moteur Android ; les PNG sources conservent leur transparence.

| Fichier livré | Fonction | Dimensions |
|---|---|---|
| `android/app/src/main/assets/art/keep.png` | Forteresse, titre, prologue et refuge | 1672 × 940 |
| `android/app/src/main/assets/art/characters.png` | Trois veilleurs et trois familles d’ennemis, atlas 3 × 2 | 1536 × 1024 |
| `android/app/src/main/assets/art/courtyard.png` | Sol peint de l’arène | 1672 × 940 |
| `android/app/src/main/assets/art/ruins.png` | Tour, barricade et ronces, atlas 3 × 1 | 2172 × 724 |

## Briefs de génération retenus

**Forteresse.** Illustration panoramique de fantasy sombre pour HEXKEEP : une forteresse ruinée allumée par quelques lanternes ambre, brume bleu pétrole, silhouette encapuchonnée avec lanterne en bas à droite. La gauche reste sombre et calme pour le titre et le récit. Peinture détaillée, profondeur atmosphérique, pas de pixels, pas de texte dans l’image.

**Personnages.** Atlas transparent de six personnages complets, grille stricte de trois colonnes et deux lignes, chaque personnage contenu dans sa cellule : gardien à capuche, épée et lanterne ambre ; éclaireur archer bleu de givre ; mystique vert avec bâton ; loup d’ombre ; spectre violet ; gardien de pierre massif. Vue de dessus légèrement inclinée, silhouettes lisibles à petite taille, art peint cohérent, aucun décor ni texte.

**Cour.** Sol panoramique d’une cour de forteresse vue de dessus, grandes dalles anciennes, ruines et végétation aux limites, lanternes chaudes sur les bords, centre ouvert. Aucun obstacle intérieur, personnage, interface ou texte. Les obstacles jouables sont ajoutés séparément pour que leurs positions correspondent aux collisions.

**Ruines — prompt final intégral.**

> Production game environment sprite atlas for HEXKEEP, an original elegant dark fantasy action adventure. Transparent alpha background. Three perfectly equal square cells in ONE horizontal row (3 columns, 1 row), all objects fully inside their own cell with generous empty transparent margin. NO labels, text, borders, interface, ground plane or scene. Each isolated object painted from high top-down three-quarter overhead camera, about 60 degrees downward, subtle warm rim light upper left, cold teal ambient shadows, sophisticated hand-painted stone textures, no pixel art, no voxel. Cell 1 left: rectangular ruined stone tower stump / carved basalt masonry block with broken crenellations, readable solid silhouette, dark grey stone with muted moss and tiny amber cracks. Cell 2 center: low rectangular ruined stone barricade, fallen engraved slabs and wooden supports, warm grey weathered surfaces. Cell 3 right: dense low patch of dark emerald brambles, silvery leaves, mossy roots, luminous teal specks, roughly rectangular footprint. Objects are gameplay cover viewed from above. Elegant restrained painterly mobile game art, high readability at small size. No elements crossing cells. Real transparent background, preserve alpha. Wide 3:1 landscape image.

## Intégration

Palette ambre, ivoire et bleu pétrole. Titres sérif, texte Android anticrénelé, panneaux translucides. Lumières, particules, oscillation des personnages, inclinaison de déplacement, traînée d’esquive, éclat d’impact et dégâts flottants sont calculés au rendu. Il s’agit de sprites peints animés par transformation, pas de personnages 3D ni d’animations dessinées image par image.

Le rendu utilise la résolution de l’écran avec un repère adaptatif de 540 unités de haut. Les anciennes fonctions M0–M7 sont présentées avec du texte natif ; le framebuffer de 240 pixels reste seulement disponible pour les outils de diagnostic historiques.

La musique est une synthèse originale à 44,1 kHz : nappes à quatre voix, motif de cloches, écho, souffle filtré et percussion progressive en combat. Les effets et la musique se règlent séparément. Un extrait reproductible est généré par `cargo run --release -p hk-apu --example score -- extrait.wav`.

## Fichiers enregistrés dans cet espace de travail

- [keep.png](../../android/app/src/main/assets/art/keep.png)
- [characters.png](../../android/app/src/main/assets/art/characters.png)
- [courtyard.png](../../android/app/src/main/assets/art/courtyard.png)
- [ruins.png](../../android/app/src/main/assets/art/ruins.png)
