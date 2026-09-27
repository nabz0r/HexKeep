# Architecture v0.4

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

Le Codex scellé détermine les paramètres de combat communs. Les achats cosmétiques, rangs, maisons et XP ne modifient pas les combats entre joueurs. Le module `hk-core/adventure.rs` applique les bonus du sac uniquement au lancement d’une expédition locale ; le lancement réseau reconstruit les combattants à partir du Codex commun. Les Mémoires affectent uniquement la garde collective ; les Murs dimensionnent la Porte. Les replays sont rejoués avant d'être admis en Haute Cour.

## Frontières de livraison

Cette version couvre un parcours DEV des fonctions M0 à M7. Les essais de laboratoire ne certifient pas un déploiement commercial : attestation Google complète, signature officielle, paiements / remboursements réels, qualification radio sur deux opérateurs et consommation sur téléphone doivent encore être traités avant production. Les limites détaillées et les preuves de test figurent dans le rapport.

## Déplacements et aventures en 0.4

`hk-sim/navigation.rs` gère rayon du personnage, déplacements balayés, glissement, correction des positions de départ, visibilité entière et recherche de chemins. L’accélération analogique est séparée de l’impulsion d’esquive. La présentation Android n’applique pas de déplacement autoritaire ; elle interpole les états à 30 Hz. Le joystick a des identifiants de pointeurs indépendants pour déplacement, attaque et visée. Les pressions d’esquive, de pouvoir et d’attaque courte sont mémorisées jusqu’au prochain tick.

`Save.journey` est ajouté avec une valeur par défaut compatible avec les sauvegardes antérieures. Il contient les objets, les trois identifiants équipés, la poussière, les lieux découverts, les contrats déjà récompensés et les dernières trouvailles. L’inventaire est limité à 60 objets ; la liste de contrats récompensés conserve 128 entrées. Les récompenses d’une sortie sont consommées lorsque l’expédition est retirée de l’état actif. La cellule d’origine est conservée pendant le combat, même si le GPS change.

Le brouillard est un masque Canvas adouci, recalculé seulement quand la découverte change. La mini-carte expose les objectifs ; le rendu et le masque suivent la caméra sans modifier les coordonnées de simulation. Les attaques ennemies sont annoncées selon les mêmes phases que leur IA.
