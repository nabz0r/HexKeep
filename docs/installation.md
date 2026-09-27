# Installer la 0.8

- **Découvrir l’édition solo** : installer `downloads/HEXKEEP-v0.8-PLAY-preview.apk`. C’est une prévisualisation signée avec une clé de test, pas une installation depuis Google Play. Android 8 minimum, ARM64 / ARMv7 / x86_64. Le jeu peut fonctionner en mode avion.
- **Mettre à jour la DEV 0.7 ou antérieure** : installer `downloads/HEXKEEP-v0.8-DEV.apk` par-dessus, sans désinstaller. Le certificat de la distribution DEV est conservé. Le test de migration vérifie identité publique, nom, XP, objets, caractéristiques, équipement et poussières.
- Les deux éditions peuvent coexister ; leurs sauvegardes sont séparées. L’édition Play n’importe pas la progression DEV.
- L’AAB est un format d’envoi, pas un fichier à ouvrir pour installer le jeu. Le candidat livré localement est non signé. La future signature Google Play peut empêcher une mise à jour de la prévisualisation : celle-ci sert au test, sans promesse de migration vers la publication.
- La progression est chiffrée dans le stockage privé. Fermer l’application suspend l’aventure ; le prochain lancement permet de la reprendre en pause. Désinstaller ou effacer les données détruit la copie locale.

Sur un téléphone avec une encoche, les boutons restent dans la zone sûre. L’indication Android de première entrée en plein écran doit être acquittée une fois. Les réglages du jeu donnent accès à l’aide, à la vie privée et aux licences.
