# Économie v0.4

Le catalogue affiche des prix TTC en euros, sans monnaie intermédiaire : pass saison 4,99 €, skins 1,99 / 3,99 / 5,99 €, lanterne 2,99 €, fanfare 1,49 €, signature 0,99 €, héraldique 3,99 €, mécénat 2,99 €/mois. Il existe 40 paliers gratuits et 40 récompenses du pass. Les combats et la veille alimentent la piste ; aucun niveau de statistiques n'est vendu.

**DEV : aucun paiement.** « Obtenir en test • 0 EUR » inscrit un reçu DEV signé, ajoute le cosmétique et permet de l'équiper. Les cosmétiques sont dessinés séparément de la simulation. Un test compare les hashes d'un combat avec et sans achat.

Google Play Billing 8 est intégré. Le client peut consulter les fiches produits. Il ne lance jamais de paiement dans la variante DEV. Le SDK transmettrait l'identifiant de Nom dans `setObfuscatedAccountId` en production. Le traitement vérifie la présence du compte dans le JSON signé ; sans preuve exploitable, l'attribution doit passer par un Édit du Trône. Les Édits GrantCosmetic / RevokeCosmetic sont effectifs uniquement au Sceau et couvrent les essais d'attribution / remboursement.

La clé de licence, les produits Play Console, la vérification complète des reçus Google et l'accès à l'API Voided Purchases ne sont pas configurés dans cette livraison. Aucun achat réel, abonnement, revenu ou remboursement bancaire n'est revendiqué. Voir [l'intégration officielle Billing](https://developer.android.com/google/play/billing/integrate).

## Butin des expéditions

Le sac et la poussière constituent une progression PvE locale, séparée du catalogue payant et des cosmétiques. Aucun équipement n’est acheté avec de l’argent réel. Les trois caches d’une sortie donnent des objets, le gardien donne une récompense supplémentaire, et les braises permettent de forger. Le recyclage rend 4 poussières par degré de rareté ; la forge coûte 30 poussières pour un objet rare. Le sac est limité à 60 objets, les récompenses en surplus étant recyclées.

Un objet peut donner vie, puissance, armure ou cadence en expédition. Ces bonus ne sont jamais injectés dans la création d’un combat réseau. Le niveau affiché, un palier par 250 éclats, n’ajoute aucune statistique cachée. La première victoire d’un contrat par cellule et période de trente minutes donne un bonus ; rejouer reste possible. Il n’y a ni échange de butin entre joueurs ni économie publique sécurisée dans cette DEV.
