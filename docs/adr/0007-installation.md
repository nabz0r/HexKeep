# ADR 0007 — APK destiné à l'installation

La v0.1 unsigned ne pouvait pas être installée directement. La DEV v0.1 avait une signature valide et avait été installée en émulateur, mais exigeait Android 9 et un système 64 bits. L'échec constaté sur les téléphones du demandeur n'a pas de message d'erreur disponible : aucune cause unique n'est affirmée.

La v0.2 principale est une variante DEV release signée, non debuggable, universelle ARM64 / ARMv7 / x86_64, Android 8 minimum. Elle conserve l'identifiant et le certificat de la DEV v0.1 pour permettre sa mise à jour. Les bibliothèques natives sont extraites à l'installation pour limiter les différences entre installateurs. Le fichier unsigned est rangé avec les artefacts de compilation et n'est pas présenté comme l'application à installer.

Les vérifications requises portent sur la signature, le manifeste, les architectures, l'installation propre, la mise à jour et le démarrage. Elles ne constituent pas un essai sur chaque modèle de téléphone.
