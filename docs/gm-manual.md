# Manuel du Trône — v0.2 DEV

## Dans l'application

Depuis la Carte, toucher **VEILLE >**, puis **TRÔNE DEV**. Android demande la biométrie ou le code si l'appareil en possède un. La DEV utilise un certificat de Trône de test inclus dans sa Genèse. Un geste de sept pressions dans le haut du titre ouvre aussi cet accès.

- **Sceller maintenant** : signe un Sceau et recalcule la racine de Merkle. Le Trône actif scelle aussi toutes les quinze minutes.
- **Édits** : choisir bannissement PvP 24 h / grâce, attribution du bastion, retour de la cellule au premier Sceau, octroi / retrait d'apparence, Proclamation. La cible DEV affichée est le premier pair connu ; sans pair, le Nom local. L'Édit reste en attente jusqu'au Sceau.
- **Haute Cour** : importer une preuve JSON issue d'un combat terminé et cosigné. Toutes les signatures, les liens d'entrées et le résultat sont vérifiés avant l'ouverture. Lecture ×0,25, ×0,5, ×1, ×2, ×4, pas à pas et statistiques par participant. Le verdict devient un Édit public.
- **Codex et monde** : publier la version suivante du Codex, fonder un sanctuaire dans la cellule, déclencher une tempête / un Sans-Nom, clôturer la saison. Les modifications attendent leur Sceau.
- **Observatoire** : nombre de cellules, événements, pairs, témoignages et générations connus. Ce n'est pas une vue omnisciente.
- **Exporter histoire** : enregistre un dossier public vérifiable avec registre, Édits, certificats et Sceaux. **Importer dossier** vérifie l'ensemble avant de remplacer l'état.

Le Trône est une autorité de finalité, pas un serveur de combat. Le réseau reçoit ses décisions signées ; un transport intermédiaire n'a aucun pouvoir de scellement.

## Cérémonie hors ligne

Construire `cargo build --release -p crown`. Aucune commande du programme n'ouvre de connexion réseau.

```sh
mkdir ceremony
crown init --root ceremony/root.json
crown genesis --root ceremony/root.json --network dev --out ceremony/genesis.json
crown certify-throne --root ceremony/root.json --public throne-public.json --out certificate.json
```

`throne-public.json` contient les 32 octets publics en tableau JSON. Le certificat est aussi produit en QR SVG. `root.json` et `crown-backup.json` sont créés en accès propriétaire uniquement ; la sauvegarde contient la phrase de 24 mots et cinq parts 3-sur-5. Les imprimer / conserver séparément hors ligne. La racine n'est jamais importée sur le téléphone.

Le projet livré possède déjà une cérémonie DEV locale dans `.dev-crown/`, exclue du dépôt et de l'archive. `assets/dev/crown.json` ne contient que sa Genèse, un certificat et la clé opérationnelle **de test**. Tous les testeurs DEV peuvent donc administrer ce même monde ; ce choix ne doit jamais servir en production. Le certificat expire après 30 jours.

```sh
crown revoke-throne --root ceremony/root.json --public throne-public.json --authority authority.json --out revoke.json
crown codex --root ceremony/root.json --network dev --file codex.json --out codex-signed.json
```

La révocation signée par la racine doit être scellée par un Trône encore autorisé. Préparer un nouveau certificat avant de révoquer le dernier Trône. Conserver toute fourche de Sceaux pour examen : le client refuse une fourche et ne choisit pas arbitrairement un gagnant.

## Interrègne et récupération

Au-delà de 72 heures sans Sceau, la Carte indique l'Interrègne ; le solo continue en provisoire. Un retour de Trône valide peut sceller le registre réconcilié. Une racine perdue n'est récupérable qu'avec sa phrase ou trois de ses cinq parts. Une nouvelle racine définit une nouvelle Genèse et un autre monde.

Pour les joueurs, le menu Identité permet de revoir les mots, certifier l'appareil et importer un dossier. Maison → Serment des Cinq permet de confier des parts chiffrées à cinq pairs co-présents et de tester une reconstruction avec trois compagnons DEV. L'essai DEV est explicitement local ; il ne prouve pas une rencontre radio réelle.

## Limites d'exploitation

La production reste verrouillée : il manque la Genèse officielle, la signature de publication et la qualification de l'attestation des appareils. La Haute Cour valide la cohérence du rejeu ; elle ne prouve pas l'absence d'assistance humaine au tir. Les paiements et remboursements réels exigent la configuration Google Play de l'éditeur.
