# Économie v0.2

Le catalogue affiche des prix TTC en euros, sans monnaie intermédiaire : pass saison 4,99 €, skins 1,99 / 3,99 / 5,99 €, lanterne 2,99 €, fanfare 1,49 €, signature 0,99 €, héraldique 3,99 €, mécénat 2,99 €/mois. Il existe 40 paliers gratuits et 40 récompenses du pass. Les combats et la veille alimentent la piste ; aucun niveau de statistiques n'est vendu.

**DEV : aucun paiement.** « Obtenir en test • 0 EUR » inscrit un reçu DEV signé, ajoute le cosmétique et permet de l'équiper. Les cosmétiques sont dessinés séparément de la simulation. Un test compare les hashes d'un combat avec et sans achat.

Google Play Billing 8 est intégré. Le client peut consulter les fiches produits. Il ne lance jamais de paiement dans la variante DEV. Le SDK transmettrait l'identifiant de Nom dans `setObfuscatedAccountId` en production. Le traitement vérifie la présence du compte dans le JSON signé ; sans preuve exploitable, l'attribution doit passer par un Édit du Trône. Les Édits GrantCosmetic / RevokeCosmetic sont effectifs uniquement au Sceau et couvrent les essais d'attribution / remboursement.

La clé de licence, les produits Play Console, la vérification complète des reçus Google et l'accès à l'API Voided Purchases ne sont pas configurés dans cette livraison. Aucun achat réel, abonnement, revenu ou remboursement bancaire n'est revendiqué. Voir [l'intégration officielle Billing](https://developer.android.com/google/play/billing/integrate).
