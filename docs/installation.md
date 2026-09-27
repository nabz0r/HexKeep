# Installer HEXKEEP v0.4

Ouvrir **HEXKEEP-v0.4.apk** sur le téléphone. Si Android le demande, autoriser l'installation depuis l'application utilisée pour ouvrir le fichier, puis confirmer Installer. Android 8 ou ultérieur est nécessaire. L'APK prend en charge ARM 32 bits, ARM64 et x86_64.

La DEV v0.1/v0.2/v0.3 signée avec la même clé peut être mise à jour sans perdre le Nom ni la sauvegarde. Conserver ses 24 mots à l'abri ; ne pas désinstaller une ancienne version avant d'avoir sa phrase de récupération. Les mots restaurent le Nom, pas une copie complète de tout le monde local.

Le fichier `app-dev-debug.apk` est destiné au diagnostic. **`app-prod-release-unsigned.apk` ne s'installe pas directement** : il attend une signature et sa production est verrouillée. Le dossier principal ne contient qu'un seul APK à choisir.

En cas d'échec : vérifier que le téléchargement fait environ 26 Mo, qu'il s'agit bien du fichier `.apk` principal, que l'espace disponible est suffisant et que la politique de l'appareil permet les installations locales. Une ancienne installation portant le même identifiant mais une autre signature bloque la mise à jour ; sauvegarder d'abord son Nom avant toute décision de désinstallation.

## Signature pour les développeurs

La DEV livrée utilise la clé de développement locale Android. Pour une publication, générer et conserver une clé dédiée, puis aligner et signer l'artefact production avec les outils du SDK :

```sh
zipalign -P 16 -f 4 app-prod-release-unsigned.apk app-prod-aligned.apk
apksigner sign --ks publication.jks --out app-prod-release.apk app-prod-aligned.apk
apksigner verify --verbose --print-certs app-prod-release.apk
```

Les mots de passe sont saisis interactivement. Signer seul ne débloque pas le réseau de production : la chaîne d'attestation et la Genèse officielles doivent être implémentées et qualifiées auparavant. Ne jamais publier avec une clé debug ou une clé racine de Couronne dans l'APK.
