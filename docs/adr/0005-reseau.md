# 0005 — P2P local de développement
Contexte : le test réel LAN est accessible sans compte ni infrastructure. CGNAT et attestation nécessitent des travaux distincts.
Décision : libp2p TCP/Noise/Yamux, mDNS et connexion IP explicite, tous les pairs calculent le combat. Pas d’autorité centrale. Les rounds sont provisoires et n’accordent pas de PR. Le premier joueur propose les participants ; tous les participants vérifient signatures et états.
Alternatives rejetées : serveur de jeu, faux compteur de joueurs, bots étiquetés comme humains, résultats déclarés scellés.
Conséquences : testable hors Internet, découverte selon les capacités du routeur. QUIC/DCUtR/relai, BLE, finalité GM et protocole de session complet restent nécessaires à la production.

## Révision v0.2

La borne de prédiction se calcule sur les entrées reçues sans trou. Un plus grand numéro reçu n'autorise jamais à dépasser une commande manquante. En attente, des demandes signées retransmettent des plages d'au plus seize entrées ; les doublons ne sont pas conservés dans la preuve. Le test de non-régression compare un flux réordonné à un flux ordonné, et l'essai mixte Android/CLI dure désormais 65 secondes.

Le relais annonce ses adresses d'écoute avant de traiter les réservations, sans attendre la réception d'Identify : la première réservation reçoit ainsi une adresse de circuit utilisable même si elle arrive immédiatement au démarrage.
