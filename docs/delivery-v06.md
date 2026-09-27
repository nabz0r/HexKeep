# Livraison 0.6 — Les Lanternes du Refuge

27 septembre 2026. Une prévisualisation solo Play et une mise à jour DEV sont livrées. **Aucune publication ni validation de Google Play n’a eu lieu.** Le bundle Play est non signé : la signature officielle reste sous le contrôle du propriétaire.

## Réalisation

La variante Play ouvre directement l’aventure, sans authentification obligatoire, GPS, Bluetooth, réseau, publicité ou achat intégré. Ses permissions se limitent à la vibration. Les sources radio et de facturation sont séparées ; la DEV garde son périmètre, et la production réseau reste verrouillée. La progression Play reste locale et distincte de la DEV.

Les aventures et le prologue sont sauvegardés avec leurs combattants, positions, vie, fioles, caches, dangers et objectifs. Une partie interrompue revient en pause ; un bilan n’est pas attribué deux fois. Un fichier chiffré illisible n’est jamais remplacé au passage en arrière-plan. Une écriture échouée reste à réessayer.

Nouveau refuge peint, couverture de boutique, icônes adaptative/monochrome, aide de jeu, retour au prologue, réglages accessibles pendant une sortie, confirmation avant abandon, prise en compte des encoches, gestion du focus audio et du retour Android 16. La confidentialité et les licences sont lisibles hors ligne.

## Validation observée

| Contrôle | Résultat |
|---|---|
| Moteur Rust | **75 tests réussis**, dont le verrou de production, la variante solo, la reprise, l’absence de double récompense, les anciennes sauvegardes et 10 000 relectures déterministes |
| Aventures DEV | **135/135 victoires** : trois royaumes × trois rôles × cinq contrats × trois difficultés |
| Aventures Play | **135/135 victoires** avec le constructeur hors ligne ; caches et inventaire conservés au rechargement |
| Android 15, téléphone 2340×1080 | Trois tests de publication et deux parcours tactiles réussis : aventure complète, butin, équipement, animation, curiosités, sauvegarde et reprise |
| Android 15, format 1280×960 | Trois tests de publication réussis ; aide et réglages inspectés visuellement |
| Android 16, AAB installé en APK fractionnés | **Six tests réussis ensemble** en 125,65 s : sauvegarde, permissions, aventure, animations, curiosités, prologue et mise en arrière-plan |
| Android 16, pages mémoire 16 Ko | **Six tests réussis** en 123,11 s sur un émulateur dédié ; `getconf PAGE_SIZE` retourne **16384** ; APK fractionnés issus de l’AAB |
| Migration DEV 0.5 → 0.6 | Installation par-dessus réussie ; identité publique, nom, XP, identifiants et statistiques d’objets, équipement et poussières conservés |
| Android Lint | Aucune erreur bloquante sur Play release ; avertissements restants sur les API historiques de rendu et les mises à jour de dépendances |
| Paquet | API cible 36, versionCode 6, trois architectures ; signature de l’APK et alignement ZIP vérifiés, neuf bibliothèques natives inspectées, segments ELF 64 bits ≥ 16 Ko |
| Boutique | Description courte 80 caractères ; couverture JPEG 1024×500, icône 512×512 et cinq captures réelles ; fiche en français |

Mesure de l’aventure tactile sur téléphone Android 15 émulé : **42,89 images/s pendant 20,87 s**, rendu SwiftShader. Ce résultat n’est pas une promesse de performance sur appareil commercial. Les essais couvrent des émulateurs à pages mémoire de 4 Ko et de 16 Ko. Aucun téléphone physique n’a été qualifié dans cette livraison.

Les rapports reproductibles sont dans [validation/v06](validation/v06/). Le protocole CI construit le bundle, inspecte les bibliothèques et lance les tests sur un émulateur API 36. L’AAB est aussi installé localement via bundletool, puis les parcours sont exécutés dans ses APK fractionnés.

## Corrections révélées par les contrôles

- La protection des écrans sensibles provoquait des mises à jour de fenêtre à chaque instant de simulation. Elle n’est désormais modifiée que si la sensibilité change ; la pause se redessine moins fréquemment.
- L’assemblage pouvait conserver d’anciennes bibliothèques dans le répertoire de sortie. La construction utilise maintenant un répertoire neuf avant remplacement ; le vérificateur refuse un moteur ne contenant pas les nouvelles fonctions.
- Le banc tactile ne comptait pas les encoches. Il utilise maintenant les coordonnées réelles de la zone de jeu. Android 16 rejetait aussi une seconde libération d’un geste déjà terminé : le pilote de test suit explicitement ce geste.
- Le test de cycle de vie attendait que le rendu soit inactif, laissant parfois un combat se terminer avant sa mise en arrière-plan. Il utilise désormais le bouton Accueil Android puis le retour à la tâche existante, et vérifie que les ticks restent figés.
- L’écran Android d’explication du mode immersif recouvrait les essais sur un émulateur neuf. L’environnement de test acquitte cette indication système ; les boutons du jeu restent testés par des événements tactiles réels.
- Les bindings Android utilisent le nettoyeur Android à partir de l’API 34 et le repli JNA avant, avec les annotations adéquates. Les services DEV ne sont plus compilés dans Play.

## Ce qui reste nécessaire

[Le dossier Google Play](play-release.md) détaille le compte, la clé d’envoi, Play App Signing, la politique publique, le contact de support, les questionnaires, la campagne sur téléphones physiques et le test fermé conditionnel de 12 personnes pendant 14 jours. Rien de cela n’est présenté comme accompli.

La localisation anglaise, TalkBack complet, les commandes remappables, le transfert de sauvegarde et le MMO public restent des évolutions. La campagne réseau à dix de la 0.5 n’a pas été répétée pour cette livraison ; les règles de combat réseau sont inchangées et les tests Rust passent, sans nouvelle revendication de qualification radio ou serveur.

## Fichiers

- [Prévisualisation Play](../downloads/HEXKEEP-v0.6-PLAY-preview.apk) : `game.hexkeep`, signature de test. Une future signature officielle peut nécessiter une nouvelle installation ; aucune migration depuis cette prévisualisation vers Google Play n’est promise.
- [DEV 0.6](../downloads/HEXKEEP-v0.6-DEV.apk) : `game.hexkeep.dev`, certificat DEV inchangé `497a8facf989aaaf6e18f0d5d7131620b96f9a11b8660d67281477b276a976dc`.
- Empreintes dans [SHA256SUMS.txt](../downloads/SHA256SUMS.txt). Bundle non signé livré localement et reconstruit par CI, distinct des APK installables.
