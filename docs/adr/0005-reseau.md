# 0005 — P2P local de développement
Contexte : le test réel LAN est accessible sans compte ni infrastructure. CGNAT et attestation nécessitent des travaux distincts.
Décision : libp2p TCP/Noise/Yamux, mDNS et connexion IP explicite, tous les pairs calculent le combat. Pas d’autorité centrale. Les rounds sont provisoires et n’accordent pas de PR. Le premier joueur propose les participants ; tous les participants vérifient signatures et états.
Alternatives rejetées : serveur de jeu, faux compteur de joueurs, bots étiquetés comme humains, résultats déclarés scellés.
Conséquences : testable hors Internet, découverte selon les capacités du routeur. QUIC/DCUtR/relai, BLE, finalité GM et protocole de session complet restent nécessaires à la production.
