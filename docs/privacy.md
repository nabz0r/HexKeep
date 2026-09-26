# Vie privée — analyse de cette alpha

Données locales : identité Ed25519, nom, préférences, royaume, rôle, cellules H3, veille et journal signé. Sauvegarde AES-256-GCM dans un fichier Android AtomicFile ; clé AES non exportable du Keystore. Si le téléphone possède un verrouillage, cette clé exige une authentification récente. Si aucun verrouillage n’est configuré, la variante dev demeure utilisable avec une clé Keystore sans authentification. Aucune sauvegarde Android automatique.

Le GPS est facultatif, demandé uniquement depuis le bouton prévu, et arrêté à la pause. Les coordonnées précises ne sont ni sauvegardées ni transmises ; le cœur ne conserve que H3 rés. 9. Les positions de combat sont des coordonnées de l’arène, sans lien avec une position réelle précise. Aucun accès aux contacts, appareil photo, micro ou stockage global.

Le LAN est désactivé jusqu’à son activation explicite dans PAIRS. Cela lève la bannière et publie une cellule grossière auprès des pairs de la Marche. L’IP est nécessaire au transport et visible des pairs. Les évènements de carte sont retardés de quinze minutes avant diffusion. Aucune Course de Mémoire et donc aucune exception de suivi rés. 10 dans cette version.

Risques résiduels : corrélation d’une cellule avec un nom, observation d’IP, modification d’un client dev, extraction sur appareil compromis. Le réseau dev n’offre pas l’anti-Sybil ou l’attestation prod. Le fichier de preuve exporté contient identités publiques, cellule et commandes de combat ; il ne contient ni mots de récupération ni coordonnées GPS exactes. L’utilisateur choisit lui-même la destination dans le sélecteur Android.

L’écran des mots est protégé contre la capture d’écran Android. Les mots sont affichés volontairement à leur propriétaire ; les noter hors ligne. Effacer les données de l’application détruit la copie locale. Désinstaller ne révoque pas les clés dont d’autres pairs possèdent déjà les événements signés.

L’affichage initial explique 16+, l’attention au monde réel, l’interdiction de jouer en conduisant et le respect des propriétés privées. Le placement réel sécurisé selon OpenStreetMap reste à faire. La carte par défaut est explicitement un terrain d’entraînement dev, pas une instruction de déplacement physique.
