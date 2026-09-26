# Couronne — état de livraison

L’alpha livrée n’a pas de Trône et ne fabrique aucun Sceau. Chaque carte indique « provisoire · aucun sceau ». `app-prod-release-unsigned.apk` reste fermé. Aucun secret de Couronne réel n’a été créé sur la machine de développement.

La cérémonie cible reste celle du prompt : racine Ed25519 hors ligne, sauvegarde 3-sur-5, Genèse, certificat de Trône expirant après trente jours, puis chaîne d’Édits et de Sceaux vérifiée par chaque pair. Ces commandes ne sont pas encore disponibles ; il ne faut pas traiter le simulateur ou un joueur comme une Couronne.

Procédures futures à implémenter et tester avant prod : révocation du Trône par la racine ; Interrègne après 72 heures sans Sceau ; récupération de la racine par trois parts ; publication d’un rollback explicite ; vérification des certificats et racines d’attestation ; Haute Cour avec rejeu et dossier conservé. La perte totale de la racine ne peut pas être résolue par un serveur de secours : aucune porte dérobée ne doit être introduite.
