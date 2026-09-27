# Notices et provenance

Le dépôt d’accueil `nabz0r/HexKeep` est publié sous AGPL-3.0. Son texte de licence et son premier commit ont été conservés. La notice MIT des versions antérieures d’HEXKEEP est conservée dans `LICENSES/HEXKEEP-pre04-MIT.txt`.

## Code et dépendances

Les composants externes conservent leurs licences respectives, indépendamment de la licence du code HEXKEEP. `Cargo.lock` et les fichiers Gradle identifient les versions résolues. Les notices disponibles dans les paquets Rust sont rassemblées dans `LICENSES/rust/`, accompagnées d’un inventaire des expressions de licence. Cette liste comprend aussi des dépendances de construction et des plateformes autres qu’Android.

- Rust et Cargo : MIT / Apache-2.0.
- Android SDK et Jetpack : licences des paquets Android, notamment Apache-2.0.
- Kotlin : Apache-2.0.
- Gradle et Android Gradle Plugin : Apache-2.0.
- JNA 5.17.0 : double licence LGPL-2.1-or-later / Apache-2.0 ; utilisation sous Apache-2.0.
- Google Play Billing 8.0.0 : conditions de licence Android SDK applicables. Les achats sont simulés dans la DEV ; aucun débit réel n’est effectué par la boutique DEV.
- libp2p, UniFFI, h3o et les bibliothèques cryptographiques : licences détaillées dans l’inventaire Cargo.

## Création artistique

Les illustrations du refuge, des personnages, des ruines et du terrain ont été générées pour HEXKEEP avec ImageGen, puis intégrées au moteur. Les prompts, dimensions et fichiers sources sont documentés dans `docs/art/v03-direction.md`. Les icônes d’objets, interfaces, masques de brume, mini-carte et effets sont dessinés par le code Android. La musique et les sons proviennent du synthétiseur original `hk-apu`.

Les fontes utilisées par l’interface sont celles de la plateforme Android ; aucune fonte commerciale n’est redistribuée.

## Références de conception

WoW, League of Legends, Dark Age of Camelot et Quake III sont cités uniquement comme références de conception. Aucun de leurs fichiers graphiques, musicaux, narratifs ni code source n’est incorporé. Les liens et les choix propres à HEXKEEP figurent dans `docs/design-v04.md`.
