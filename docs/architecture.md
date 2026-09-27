# Architecture v0.2

Le cœur de combat est Rust : calcul entier Q24.8, horloge logique 30 Hz, RNG ChaCha8 déterministe. Android fournit une présentation Canvas accélérée, AudioTrack 44,1 kHz, les entrées tactiles, GPS, Bluetooth, Keystore, biométrie, fichiers et service de veille. La simulation possède sa boucle à 30 Hz ; le rendu suit les images Android avec interpolation. Les textes et illustrations sont dessinés à la résolution de l’écran, sur un repère adaptatif de 540 unités de haut. Les commandes sémantiques du cœur permettent de conserver les écrans avancés sans rasterisation rétro.

## Modules

| Module | Responsabilité |
|---|---|
| hk-proto | Types, coordonnées, domaines et hashes |
| hk-sim | Combat, IA, évolution, Codex et siège |
| hk-world | H3 9/8/7, veille, Noir, bastions, garde fixe |
| hk-crypto | Ed25519, BIP39, Shamir, P-256, X25519/AEAD, témoignages |
| hk-ledger | Événements signés, DAG et fold déterministe |
| hk-net | libp2p, rollback, entrées chaînées, preuves et rejeu |
| hk-crown | Genèse, certificats, Édits, Sceaux, Merkle, réconciliation |
| hk-season | Mémoires, garde collective, saison, cosmétiques et Maisons |
| hk-core | Parcours du jeu, écrans, état persistant et raccordement |
| hk-ppu / hk-apu | Commandes de présentation, diagnostic raster et musique ambiante originale |
| hk-ffi | Frontière UniFFI, accès synchronisés au cœur |
| crown | Cérémonie entièrement hors ligne |
| hexkeep-relay | Circuits réseau aveugles et temporaires |
| hexkeep-sim | Pairs indépendants, rejeu, déterminisme et export statistique |

Aucun serveur n'est consulté pour décider d'un résultat. Les participants simulent et cosignent ; le Trône décide de la finalité du registre qu'il connaît. Les réplicas refusent un Sceau dont ils ne possèdent pas les événements nécessaires au recalcul.

## Stockage et cycle Android

L'état v0.1 est migré par ajout des champs de campagne. Le Nom est conservé. Snapshot JSON chiffré AES-GCM avec clé non exportable Android Keystore, écriture AtomicFile. Sur appareil sécurisé, le déverrouillage passe par biométrie ou code. Une corruption ne provoque pas de remise à zéro silencieuse. Le DAG signé est conservé dans ce snapshot ; un journal physique append-only séparé n'est pas encore utilisé.

Les sessions de trafic sont éphémères, certifiées par la P-256 de l'appareil. La racine de Couronne reste hors téléphone. La DEV contient une délégation de test partagée, explicitement impropre à la production.

Sans Phare, mise en pause = arrêt audio/GPS/réseau, sauvegarde et interruption honnête du combat. Le Phare garde une simulation et un service réseau visibles pendant la pause, sur secteur/Wi-Fi, avec ancre locale. Aucune boucle en double entre activité et service.

## Séparation du combat et de la progression

Le Codex scellé détermine les paramètres de combat communs. Les achats, rangs, maisons et XP ne modifient pas la simulation. Les Mémoires affectent uniquement la garde collective ; les Murs dimensionnent la Porte. Les replays sont rejoués avant d'être admis en Haute Cour.

## Frontières de livraison

Cette version couvre un parcours DEV des fonctions M0 à M7. Les essais de laboratoire ne certifient pas un déploiement commercial : attestation Google complète, signature officielle, paiements / remboursements réels, qualification radio sur deux opérateurs et consommation sur téléphone doivent encore être traités avant production. Les limites détaillées et les preuves de test figurent dans le rapport.
