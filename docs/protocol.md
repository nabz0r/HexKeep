# Protocole v0.5 DEV

`hk-proto`, `hk-net`, `hk-crypto/identity`, `hk-ledger`, `hk-crown` et `hk-season` définissent les formats exacts. La sérialisation signée est Borsh, avec entiers little endian. JSON sert à l'export et à la sauvegarde, pas à construire arbitrairement les octets signés.

Chaque signature est préfixée par `HEXKEEP/v1/<network>/<type>\0`. Types : event, device, session, net, witness, recovery-share, house-oath, test-receipt, genesis, throne, edict, seal, codex. Les exports ont leur propre numéro de version ; le domaine crypto v1 ne signifie pas compatibilité avec les paquets applicatifs v0.1.

## Identité et transport

Nom Ed25519 → liaison de clé P-256 signée par le Nom → certificat de session Ed25519 signé P-256, durée maximale 86 400 s. Android crée la P-256 dans son Keystore avec défi d'attestation BLAKE3(Nom public), StrongBox si disponible, repli DEV sur Keystore sans attestation quand l'émulateur ne le permet pas. La chaîne de certificats est jointe. Le transport vérifie les signatures et les bornes de session ; la qualification Google de production n'est pas activée.

`Packet { author: Nom, sequence, message, signature, certificate?, issued }`. Les paquets actuels doivent avoir un certificat valide et une date à ±120 s de la réception. Le Nom public reste l'identifiant ; les signatures haute fréquence utilisent la session. Les certificats répétés sont factorisés dans les exports de preuves, puis restaurés avant la vérification des octets signés.

TCP/Noise/Yamux, QUIC, mDNS, gossipsub, Identify, Ping, AutoNAT, DCUtR, Kademlia et client Relay v2. Gossip par Marche `hk/dev/marche/<H3>/v05`, et canal de Mémoires consenti `hk/dev/v05/memories`. Limite de paquet 256 KiB, déduplication et tailles de listes contrôlées. Le relais a des limites de réservations, de durée et d'octets ; il ne lit pas les charges Noise entre les clients.

## Combat et preuve

StartV2 transporte cellule, participants, nonce, mode, Codex et Murs. Tous dérivent le même Champ, appliquent la même configuration et calculent leur propre état. Les entrées sont retardées de deux ticks ; la prédiction tolère une fenêtre de rollback de huit ticks. La limite d'avance utilise les entrées reçues sans trou pour chaque joueur, même si les paquets arrivent dans le désordre. `MissingInputs { room, player, from, to }` demande au propriétaire de retransmettre au plus seize entrées ; les doublons ne grossissent pas le dossier de preuve. Les résultats finaux sont retransmis pendant l'attente des autres signatures. Les conflits d'entrée signée et divergences d'état interrompent la validation.

Chaque entrée référence le hash de la précédente du même participant : BLAKE3/Borsh de `(room, Nom, tick, input, previous)`, origine nulle au tick 3. Les hashes confirmés sont échangés toutes les 30 trames de simulation. Une preuve complète nécessite la signature du même résultat par chaque participant. Le lecteur exige toutes les entrées, vérifie les liens puis rejoue jusqu'au hash final. Un combat interrompu reste exportable mais ne passe pas pour une preuve complète.

## Registre, Couronne et finalité

Événement : version, réseau, cellule, parents triés (maximum 8), référence de Sceau, Lamport, type, royaume, valeur, heure, Nom auteur, signature. Fold dans l'ordre `(Lamport, hash)` ; signatures invalides, parents absents, liens entre cellules et auteurs bannis sont rejetés. Fusion par union de hashes. L'inventaire tourne par pages de 512 IDs ; les manquants sont demandés par lots de 128. L'historique public conserve le délai de quinze minutes.

La Genèse signe réseau, racine, Codex, relais et début de saison. La racine certifie les Trônes pour trente jours. Les Édits forment une chaîne de séquences consécutives. Ban, Unban, Rollback, AssignBastion, GrantCosmetic, RevokeCosmetic, Verdict, Proclamation, Foundation, WorldEvent, Codex, SeasonKey, RelayList, RevokeDevice et RevokeThrone ont des types explicites.

Un Sceau signe époque, prédécesseur, heure, racine de Merkle des cellules, tête des Édits, version Codex, certificat de Trône et ensemble d'événements couverts. Arbre binaire creux indexé sur 64 bits H3 ; domaines séparés pour feuille vide, feuille et branche. La réception exige le registre nécessaire et recalcule la racine. Les forks sont refusés. Le rollback relit un Sceau antérieur identifié publiquement ; il n'efface pas l'histoire.

## BLE, récupération, économie

Balise = 16 premiers octets de BLAKE3(session publique || époque de dix minutes), plus royaume. Un témoignage signé lie observateur, sujet, jeton observé, heure et cellule. Une co-présence requiert deux témoignages réciproques récents. Les parts Shamir 3-sur-5 sont chiffrées X25519 / ChaCha20-Poly1305 pour chaque dépositaire et l'enveloppe est signée par le Nom propriétaire.

Les Maisons ont une héraldique 16×16 et des serments Ed25519 distincts. La fondation DEV avec personnages de test est marquée explicitement. Les reçus de boutique DEV sont signés dans un domaine distinct des vrais reçus Play et ne représentent aucun paiement. Les structures de campagne / cosmétique ne sont pas consultées pour calculer les dégâts, PV ou déplacements du combat réseau. Les bonus locaux du sac sont appliqués uniquement par le lancement d’expédition PvE.

La validation des chaînes de session est mise en cache par empreinte complète, dans une table bornée à 256 entrées. Une modification du certificat impose une nouvelle vérification. La signature de chaque paquet et ses bornes temporelles sont toujours vérifiées ; le moteur ne revalide pas une seconde fois un paquet déjà authentifié par le transport. Les essais incluent les signatures modifiées et certificats expirés après amorçage du cache.

## Compatibilité de simulation

La 0.5 corrige la sortie des angles de mur dans la navigation. Cette modification change les trajectoires calculées et impose une version commune du moteur. Identify utilise `/hexkeep/dev/5` et les topics géographiques sont suffixés `v05`. Tous les participants doivent employer la même version. Une preuve d’une version antérieure demande son ancien moteur ; la preuve `examples/cosigned-duel.json` livrée sur `main` est une preuve 0.5. Les sauvegardes de profil migrent par ajout de champs, sans changer le Nom ni le registre.
