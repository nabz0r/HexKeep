# 0001 — Livraison de développement vérifiable
Contexte : aucun certificat APK de production, racine de Couronne, compte Play ni appareil attesté ne sont fournis. Le premier APK est le réseau dev, isolé par le nom de paquet et les signatures.
Décision : progresser M0→M7 et ne jamais présenter comme prêt un jalon non validé. Le mode prod reste fermé tant que l’attestation complète et la Genèse officielle ne sont pas configurées. Aucune racine de production n’est générée silencieusement.
Alternative rejetée : accepter automatiquement tout appareil en prod ou simuler des achats, des pairs ou des Sceaux.
Conséquences : l’APK dev est installable directement. Les déplacements sans GPS sont explicitement des déplacements de développement. Aucun paiement réel.
