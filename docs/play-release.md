# Google Play — dossier de lancement 0.6

État au 27 septembre 2026 : **candidat technique pour les tests**, pas une application publiée ni une approbation de Google. Le paquet `game.hexkeep` est une édition solo hors ligne. L’ambition MMO géolocalisée reste dans la DEV `game.hexkeep.dev`, avec son verrou de confiance production intact.

## Ce qui empêchait la publication

La variante `prod` de la 0.5 ouvrait un écran de production verrouillée ; aucun AAB de lancement n’était livré. Les fonctions radio, les permissions sensibles, la boutique de développement et les ambitions réseau dépassaient le périmètre validé. Une aventure ne survivait pas à la destruction du processus. La politique de confidentialité décrivait une alpha réseau, et le matériel de boutique n’était pas préparé.

La 0.6 ajoute une variante `play`, un AAB reproductible, un accès direct au solo, une sauvegarde du combat, des sources Android séparées pour le réseau et la facturation, une politique propre à cette édition et les ressources du dossier `store/`. Le téléphone n’a besoin d’aucune permission sensible. Les réglages, la confidentialité, les crédits et toutes les licences tierces sont lisibles hors ligne.

## Derniers jalons avant mise en ligne

| Priorité | Travail | État / condition de sortie |
|---|---|---|
| P0 | Compte développeur, identité vérifiée et fiche application | À effectuer dans le compte du propriétaire ; aucun accès Play Console n’a été utilisé |
| P0 | Identifiant définitif, clé d’envoi et Play App Signing | `game.hexkeep` proposé ; AAB livré non signé. Ne jamais utiliser la clé de prévisualisation comme clé officielle |
| P0 | Politique publique et contact | Texte livré dans `docs/privacy-play.md` et dans le jeu. Héberger une URL publique stable, active et non géobloquée ; renseigner l’adresse de support réelle du propriétaire |
| P0 | Déclarations Play Console | Data safety, publicité absente, accès sans identifiants, public cible, questionnaire IARC sur les combats fantastiques. Répondre pour l’ensemble des versions distribuées sous cet identifiant |
| P0 | Tests internes sur téléphones | Installer les APK fractionnés issus de l’AAB, tester Adreno/Mali, Android 8/13/15/16, 4 Go RAM, mode avion, appels, rotation, veille, manque d’espace, installation et mise à jour |
| P0 conditionnel | Test fermé obligatoire | Pour les comptes personnels concernés : au moins 12 testeurs inscrits continuellement pendant 14 jours, puis demande d’accès à la production |
| P1 | Qualité mesurée par les joueurs | Aucun crash bloquant, aucune perte de sauvegarde ; consigne comprise et première victoire atteinte sans assistance ; confort de visée, lisibilité et échauffement acceptables |
| P1 | Rapport de pré-lancement Google | Traiter les crashs, ANR, problèmes de compatibilité et préconisations réellement applicables avant ouverture publique |
| P1 | Déploiement initial maîtrisé | Fiche française honnête, disponibilité initiale cohérente avec la langue, surveillance des vitals et du retour qualitatif ; élargissement après correction |

Les émulateurs et simulations ne prouvent pas les performances de téléphones commerciaux, la rétention ou l’agrément pour un nouveau joueur. Les critères ci-dessus ne sont pas marqués terminés sans observation.

## Exigences vérifiées dans les sources officielles

- Depuis le 31 août 2026, une nouvelle application téléphone doit cibler **Android 16 / API 36**. La 0.6 cible 36. [Google Play](https://support.google.com/googleplay/android-developer/answer/11926878?hl=en).
- Le code natif destiné à Android 15+ doit supporter les pages mémoire **16 Ko**. Le contrôle `scripts/verify-play.py` inspecte les segments ELF 64 bits, l’alignement de l’APK, les permissions et le manifeste du bundle. Un essai sur système 16 Ko reste distinct de cette inspection. [Android Developers](https://android-developers.googleblog.com/2025/05/prepare-play-apps-for-devices-with-16kb-page-size.html).
- Une politique accessible dans le jeu et par URL, ainsi qu’une déclaration Data safety exacte, sont nécessaires même sans collecte. [Politique User Data](https://support.google.com/googleplay/android-developer/answer/10144311?hl=en).
- Le test fermé de 12 personnes pendant 14 jours dépend du type et de la date de création du compte ; il ne s’applique pas indistinctement à tous les comptes. [Exigences de test](https://support.google.com/googleplay/android-developer/answer/14151465?hl=en).
- Icône 512×512 et visuel de présentation 1024×500 ; les captures fournies proviennent du jeu exécuté. [Ressources de boutique](https://support.google.com/googleplay/android-developer/answer/9866151?hl=en-GB).

## Construire et signer

`scripts/build.sh` produit l’APK DEV signé localement, la prévisualisation Play signée avec la clé debug locale et l’AAB Play. Le bundle est **non signé** quand les variables ci-dessous ne sont pas définies. L’application Play n’affiche ni paiement, ni réseau, ni accès au Trône ; le paquet `prod` conserve son verrou.

Pour une construction officielle, fournir hors dépôt `HK_UPLOAD_STORE` (chemin absolu), `HK_UPLOAD_STORE_PASSWORD`, `HK_UPLOAD_ALIAS`, `HK_UPLOAD_KEY_PASSWORD`. Les quatre doivent être présents ensemble. Ne jamais consigner les valeurs dans des journaux. Dans `android/`, lancer `./gradlew bundlePlayRelease`. La clé privée reste sous le contrôle du propriétaire. Les versions suivantes devront augmenter `versionCode`.

Avec JDK 17, Android build-tools 36 et [bundletool](https://developer.android.com/tools/bundletool) :

```sh
python3 scripts/verify-play.py \
  --apk artifacts/HEXKEEP-v0.6-PLAY-preview.apk \
  --aab artifacts/HEXKEEP-v0.6-PLAY.aab \
  --bundletool /chemin/bundletool.jar
```

Ajouter `--require-signed` uniquement au bundle signé pour l’envoi. L’outil refuse une clé Android Debug. Il ne certifie pas l’identité du signataire ni une acceptation par Google. Générer ensuite un ensemble d’APK avec `bundletool build-apks` et tester son installation avant l’envoi à la piste interne.

## Après le lancement solo

Une traduction anglaise complète et relue, des menus accessibles aux lecteurs d’écran, la personnalisation des commandes et un transfert chiffré de progression sont les prochains investissements de confort. L’interface de la 0.6 reste française et son Canvas ne fournit pas encore une navigation TalkBack complète.

Pour publier le MMO géolocalisé : hébergement et exploitation réseau pérennes, chaîne d’autorité/attestation, sûreté des lieux réels, consentements et politique adaptés, lutte contre la triche, signalement/modération, essais de charge et coût d’exploitation. Un réseau de dix pairs de développement ne valide pas ces exigences. Il faut qualifier ce produit séparément ; ne pas réactiver simplement les menus DEV dans l’édition Play.
