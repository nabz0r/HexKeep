# HEXKEEP v0.2 — La Première Nuit

Livraison DEV du 27 septembre 2026. L'APK principale est `HEXKEEP-v0.2.apk`, signée, sans indicateur de débogage, version 2 / 0.2.0-premiere-nuit, Android 8 minimum, ARM64, ARMv7 et x86_64. La v0.2 ouvre les parcours DEV des jalons M0 à M7. Elle ne constitue pas une qualification de production ni une affirmation de compatibilité avec tous les téléphones.

## Installation corrigée

L'APK unsigned n'est pas installable : Android exige une signature. L'ancienne DEV excluait Android 8 et les téléphones ARM 32 bits. La nouvelle livraison contient les trois architectures, utilise une signature vérifiée et conserve l'identifiant `game.hexkeep.dev` et le certificat de la DEV précédente pour permettre sa mise à jour. L'extraction des bibliothèques natives est activée. Une erreur de résolution réseau propre à Android, qui empêchait le démarrage du pair, a aussi été corrigée.

Installation neuve et mise à jour depuis la v0.1 ont réussi sur les émulateurs Android disponibles. Le motif exact des échecs sur les téléphones de l'utilisateur reste inconnu : aucun de ces téléphones n'est connecté au banc de test.

## Jalons livrés

| Jalon | Présent et vérifié dans la DEV | Qualification restante |
|---|---|---|
| M0 | Workspace Rust, Android Kotlin, UniFFI, rendu pixel, audio original, automatisation de construction. | La CI distante n'a pas été exécutée. |
| M1 | Neuf combinaisons royaume/rôle, combat, esquive, compétences, IA, tutoriel, évolution, Codex réglable ; 10 000 combats complets déterministes. | Équilibrage et accessibilité sur téléphones physiques. |
| M2 | Monde H3, veille, Noir, bastions, garde collective, carte et sauvegarde chiffrée persistante. | Le stockage est un snapshot chiffré ; placement sûr OSM et journal disque append-only non réalisés. |
| M3 | Nom Ed25519, BIP39, appareil P-256 Android Keystore, session Ed25519 24 h, biométrie/code, Shamir 3-sur-5 et parts chiffrées. | Vérification complète des racines d'attestation Google et restauration entre trois gardiens humains non qualifiées. |
| M4 | TCP/Noise, QUIC, mDNS, gossip, synchronisation paginée, duel, chaînes d'entrées, preuve cosignée et rejeu vérifiable. | Découverte radio/mDNS sur de vrais téléphones à mesurer. |
| M5 | Dix participants, rollback limité, siège Porte/Cour, Courses de Mémoire et chute, BLE tournant, Phare, relais v2/DCUtR, fusion de registres. | Deux opérateurs 4G, partition réseau réelle de 30 min et rencontres BLE physiques non testés. |
| M6 | CLI crown hors ligne, Genèse, certificats de Trône, 15 types d'Édits, Merkle, Sceaux, Ban/Rollback, Haute Cour avec lecture de preuve et verdict. | Chaîne Parjure de production, révocation d'appareil en réseau et procédures de racine sur matériel isolé incomplètes. |
| M7 | Saison I, 40 paliers gratuits et 40 de pass, Nuit Longue, Chronique scellée, catalogue DEV, Maisons, héraldique 16×16, apparences équipables, intégration Billing 8. | Boutique gratuite de test ; validation des achats/licences et remboursements Play, fondation de Maison à trois humains et diffusion complète des cosmétiques restent à terminer. |

L'entraînement, la lisibilité des objectifs, le mouvement et la progression purement cosmétique s'inspirent de principes documentés chez de grands jeux. Les personnages, graphismes et musiques restent ceux de HEXKEEP ; voir `design-heritage.md`.

## Validation

49 tests Rust sont verts. Deux essais de 65 secondes avec deux Android et huit pairs donnent respectivement 44 et 54 checkpoints communs identiques. Le duel par relais se termine avec deux signatures et se rejoue à l'identique. Le solo hors ligne, le Keystore, la boutique/maison/Trône et le Phare passent sur la dernière APK. Le déterminisme de 10 000 combats complets concorde en ARM64, x86_64 et Android ARM64.

Aucun test automatisé final n'est rouge. Les essais physiques et de production qui n'ont pas pu être exécutés restent explicitement non qualifiés.

Les résultats mesurés, empreintes et commandes sont consignés dans `test-protocols.md`. Les fichiers de preuve et journaux finaux accompagnent la livraison dans `validation/`. Un scénario non exécuté n'est pas compté comme réussi. Les échecs intermédiaires ont servi à corriger le code ; ils ne sont pas effacés des journaux de travail.

## Écarts et limites

- Le monde DEV inclut une délégation de Trône partagée et des compagnons de test explicitement indiqués. Le certificat expire après trente jours. La racine privée n'est ni dans l'APK ni dans le dépôt.
- Les modes de déplacement DEV permettent le test sans GPS. Ils ne prouvent aucune présence physique. Les Maisons et le Serment des Cinq ont un parcours local de démonstration, distinct des primitives cryptographiques réelles.
- Le rejeu prouve la cohérence du combat et ses signatures, pas l'absence d'assistance au tir. L'échange historique applique un délai de confidentialité ; un pair privé de l'historique complet d'un Sceau le refuse provisoirement.
- La traduction anglaise est partielle. L'orchestration audio, les catégories de cosmétiques et leurs interactions sociales ne couvrent pas chaque détail du prompt maître. Les statistiques sont des observations locales, pas une mesure de rétention ou de revenus fictive.
- Aucune publication Play, aucun paiement et aucun serveur d'autorité public ne sont actifs. La production reste fermée. Le fichier unsigned demandé est livré pour les développeurs uniquement.

## Risques ouverts, par gravité

**Critique avant une production ouverte :** attestation officielle, identification de l'APK autorisée, révocation complète des appareils, contrôle des licences/achats et autorité de Couronne non partagée. La DEV ne doit pas être promue en production en changeant simplement un drapeau.

**Élevé avant un pilote public :** audit anti-Sybil/DoS, quotas sous charge, reconnexions et NAT mobiles réels, concurrence de plusieurs Trônes, récupération sociale complète sur plusieurs téléphones. Un abandon ou une déconnexion ne fournit pas une preuve de fin cosignée complète.

**Moyen :** batterie, Bluetooth, biométrie matérielle, formats d'écran atypiques, accessibilité, anglais complet et équilibre. Les mesures AVD ne valent pas une garantie de 60 i/s sur un téléphone de 2021.

Les sources synchronisées du projet sous `sources/` n'ont pas été modifiées.
