# Modèle de menace

## Fonctionnel dans cette version
- Rejeu entre domaines : signatures séparées `dev/event`, `dev/net`. Prod fermé.
- Altération de sauvegarde : AES-GCM et écriture atomique ; erreur visible, aucune remise à zéro silencieuse.
- Modification de messages : Ed25519 strict, transport Noise, auteurs de combat vérifiés contre les participants.
- Résultats contradictoires : empreintes périodiques, rejeu, preuves cosignées ; aucune récompense réseau en dev.
- GPS trop rapide ou imprécis : filtre local, sans prétendre qu’il constitue une attestation.
- Achats : absents. Aucune voie de puissance payante.

## Bloquants de production
Critique : absence de chaîne d’attestation Android hors ligne, certificats P-256/session, Couronne et Sceaux. Un client dev peut mentir sur sa présence et son historique. Ne pas utiliser le réseau dev comme guerre territoriale canonique.

Élevé : résistance aux attaques Sybil/éclipse et au déni de service non auditée ; aucun transport mobile Internet validé ; la synchronisation est limitée à de petits historiques locaux ; le journal chargé en mémoire n’est pas un stockage append-only compacté durable sous Sceau.

Moyen : paramètres de combat non équilibrés sur un panel humain ; un pair interrompu produit une preuve incomplète ; pas de reprise de combat après fermeture ; pas de témoin BLE.

Les signatures prouvent un auteur, pas la véracité de son récit. Un hash identique prouve un calcul identique pour les entrées données, pas l’absence d’assistance de visée. Aucune protection anti-triche absolue n’est revendiquée.

## État v0.2

Les signatures de session, expirations, parts chiffrées, chaînes d'entrées, rejeux complets, chaînes d'Édits, roots Merkle, bans et rollbacks ont des tests exécutables. Les imports de finalité sont transactionnels ; une autre Genèse, une fourche ou un registre incomplet sont refusés.

La clé opérationnelle de Trône DEV est volontairement distribuée aux testeurs. Elle n'offre donc **aucune séparation d'administration entre testeurs**. La racine n'est pas dans l'APK. La production est fermée plutôt que d'accepter des appareils non qualifiés.

La verification P-256 prouve la possession de la clé liée au Nom ; elle ne remplace pas la validation Google de la chaîne d'attestation, du démarrage vérifié, de la signature APK et des révocations. Cette validation de production, le durcissement complet contre l'abus de trafic et l'analyse anti-assistance restent des travaux distincts. Les bots et GPS fictifs permis en DEV ne sont pas des preuves d'équité sur appareils adversariaux.
