# HEXKEEP v0.3 — Les Braises

Livraison du 27 septembre 2026. Variante DEV jouable, avec les fonctions M0–M7 conservées. Nouvelle présentation, nouveau départ et nouvelle expédition.

## APK à installer

- **HEXKEEP-v0.3.apk**, signé, paquet `game.hexkeep.dev`, versionCode **3**.
- Android **8+**, ARMv7 / ARM64 / x86_64 ; application release non débogable.
- Taille : **26,487,557 octets**, soit **26.49 Mo**.
- SHA-256 : `fe134e584c8bc8144de6e70847b8161d6c840d99c7cb5b890cb69ecd2c58b16d`.
- Certificat identique à la v0.2. Installation de remplacement validée, identité publique, Nom et progression conservés.
- Signatures et alignement de l’APK contrôlés. Bibliothèques 64 bits alignées sur 16 Kio ; ARMv7 sur 4 Kio ou plus.

## Ce qui change

Un prologue en trois scènes précède le choix du veilleur et l’apprentissage jouable. Le refuge présente l’expédition, la carte, la forteresse et la coopération. Les 24 mots restent accessibles depuis les réglages, sans bloquer les premières minutes.

Quatre illustrations originales remplacent les visuels rétro : forteresse, atlas de six personnages, sol peint et atlas de ruines. Le texte et les boutons sont dessinés à la résolution de l’écran. Déplacement interpolé, inclinaison, traînées d’esquive, lumière, impact et dégâts flottants donnent du mouvement aux sprites. Trois plans d’obstacles utilisent le même décor de cour, avec des variations de teinte.

L’expédition demande de rallumer trois balises, repousser les renforts et battre un gardien. Elle laisse trois vies et rapporte une progression au refuge. Les ennemis poursuivent, préparent une salve ou combattent à distance selon leur rôle. Leurs attaques sont annoncées visuellement.

Les deux pouces sont suivis indépendamment. La visée assistée peut être désactivée ; la visée manuelle, l’esquive et le pouvoir restent disponibles ensemble. Les petits formats utilisent un cadrage adapté. Un réglage renforce le contraste.

La musique est recomposée : nappes, cloches, écho, souffle filtré et percussion progressive. Elle fonctionne à 44,1 kHz et se règle séparément des effets et des vibrations. Aucun service de musique ou de génération n’est appelé par l’APK.

## Validation

| Vérification | Résultat |
|---|---|
| Tests Rust du workspace | **55 réussis** |
| Expéditions avec les commandes réelles | **9/9 réussies**, trois royaumes × trois rôles, trois plans d’obstacles |
| Déterminisme | **10 000 combats complets**, même empreinte ARM macOS, x86_64 macOS et Android ARM64 |
| Parcours Android | Prologue, héros, deux doigts, esquive, pouvoir, carte, pause, reprise et sauvegarde validés |
| Suite Android avancée | Campagne, cosmétiques, héraldique, Trône, Keystore et Phare validés |
| Écrans | **2340 × 1080** et **1440 × 1080 / 4:3** ; contrôle visuel des captures |
| Mise à jour | APK v0.2 → v0.3, même identité et progression |
| Dix participants | **Deux Android + huit moteurs natifs**, **55 points de contrôle identiques**, jusqu’au tick **1650** |
| Relais aveugle | Duel complet, **71 points de contrôle concordants**, preuve rejouée à **2133 ticks**, deux cosignataires |
| Musique | Extrait de 24 s, 44,1 kHz mono, crête 0,287, RMS 0,037, absence de saturation numérique |

Empreinte de la suite déterministe :
`315e9f9af6bf5a99883803dc345224ff82aa4c23feaa9473eaa6c9b7e3ffd2d0`

La mesure de rendu solo sur l’émulateur Pixel 5, en 2340 × 1080 avec GPU logiciel, donne **46.4 images/s** pendant **12.02 secondes**. La simulation possède une boucle séparée à 30 Hz. Cette mesure ne vaut pas certification à 60 images/s sur téléphone.

Les tests d’installation et de jeu utilisent deux émulateurs Android 15 ARM64. Aucun téléphone physique n’était connecté. Autonomie, température, audio entendu sur haut-parleur réel, Bluetooth entre téléphones et réseaux mobiles d’opérateurs restent à qualifier. Le résultat livré est une version DEV ; la production, l’attestation officielle et les paiements réels restent verrouillés comme dans la v0.2.

La validation du Phare exige le Wi-Fi connecté, l’alimentation et la permission de localisation. Le service s’arrête lorsque ces conditions ne sont plus réunies. Le premier essai sans Wi-Fi a confirmé cet arrêt ; le scénario avec les conditions requises passe.

Les anciens salons v0.2 sont séparés des salons v0.3, car les cartes et l’IA changent. Les anciennes preuves de combat demandent la simulation de leur version ; les identités et la progression locale sont migrées.

Les journaux, captures et preuves sont joints dans le dossier de livraison. Les instructions de construction, la direction artistique, les fichiers des images, les briefs et le prompt intégral des ruines sont dans les sources et dans `DIRECTION-ARTISTIQUE.md`.
