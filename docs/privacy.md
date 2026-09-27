# Vie privée — analyse de cette alpha

Données locales : identité Ed25519, nom, préférences, royaume, rôle, cellules H3, veille et journal signé. Sauvegarde AES-256-GCM dans un fichier Android AtomicFile ; clé AES non exportable du Keystore. Si le téléphone possède un verrouillage, cette clé exige une authentification récente. Si aucun verrouillage n’est configuré, la variante dev demeure utilisable avec une clé Keystore sans authentification. Aucune sauvegarde Android automatique.

Le GPS est facultatif, demandé uniquement depuis le bouton prévu, et arrêté à la pause. Les coordonnées précises ne sont ni sauvegardées ni transmises ; le cœur ne conserve que H3 rés. 9. Les positions de combat sont des coordonnées de l’arène, sans lien avec une position réelle précise. Aucun accès aux contacts, appareil photo, micro ou stockage global.

Le LAN est désactivé jusqu’à son activation explicite dans PAIRS. Cela lève la bannière et publie une cellule grossière auprès des pairs de la Marche. L’IP est nécessaire au transport et visible des pairs. Les évènements de carte sont retardés de quinze minutes avant diffusion. La Course de Mémoire demande un consentement distinct avant de publier la cellule de transport.

Risques résiduels : corrélation d’une cellule avec un nom, observation d’IP, modification d’un client dev, extraction sur appareil compromis. Le réseau dev n’offre pas l’anti-Sybil ou l’attestation prod. Le fichier de preuve exporté contient identités publiques, cellule et commandes de combat ; il ne contient ni mots de récupération ni coordonnées GPS exactes. L’utilisateur choisit lui-même la destination dans le sélecteur Android.

L’écran des mots est protégé contre la capture d’écran Android. Les mots sont affichés volontairement à leur propriétaire ; les noter hors ligne. Effacer les données de l’application détruit la copie locale. Désinstaller ne révoque pas les clés dont d’autres pairs possèdent déjà les événements signés.

La v0.3 commence par un prologue fictif jouable hors ligne. Le GPS est activé seulement depuis la carte, après la permission Android. Le déplacement DEV reste un outil de test. Le placement réel sécurisé selon OpenStreetMap reste à faire ; la carte DEV ne constitue pas une instruction de déplacement physique.

## Compléments v0.2

Le Bluetooth diffuse un jeton de session tournant et un royaume. La clé de Nom n'est pas mise dans la balise. Les témoignages signés restent soumis au contrôle de co-présence. Le refus de permission conserve l'accès au solo. La Course demande un consentement explicite avant diffusion d'une cellule H3 de résolution 10. Aucun point GPS exact n'est envoyé au réseau.

Le Phare utilise une notification persistante et une commande Arrêter ; le service exige secteur et Wi-Fi et conserve une ancre GPS locale. Son démarrage conserve le choix de bannière. Il n'y a pas de permission de localisation permanente. La boutique DEV ne débite aucun paiement et ne recueille pas de coordonnées bancaires ; l'interrogation facultative du catalogue utilise le SDK Google Play Billing.

Les preuves de combat incluent des positions dans l'arène fictive, pas les coordonnées physiques. Les trois preuves complètes les plus récentes sont conservées localement ; l'export de dossier est une action du joueur. Les certificats publics lient les sessions au Nom auprès des pairs de la partie. La DEV ne doit pas être utilisée comme outil de suivi de personnes réelles.

## Présentation v0.3

Le Nom est généré localement à la première entrée dans le prologue. La récupération par 24 mots est accessible dans les réglages ; le jeu rappelle de les noter, sans imposer le questionnaire avant de jouer. Les scènes, sprites et la musique sont intégrés dans l’APK : aucun service de génération n’est contacté par le téléphone. Les préférences d’assistance de visée restent dans le stockage privé de l’application.
