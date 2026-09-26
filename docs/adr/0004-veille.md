# 0004 — Veille au premier plan seulement
Contexte : les types foreground-service d’Android 14/15 et le Phare exigent des justifications et tests de consommation réels.
Décision : cette alpha veille seulement lorsqu’elle est au premier plan. GPS et réseau s’arrêtent à la pause. Aucun service de fond simulé, aucune permission de localisation en arrière-plan.
Alternative rejetée : afficher « Phare actif » sans service attesté ni mesures, ou garder des sockets invisibles après fermeture.
Conséquences : le Phare et la veille écran éteint sont explicitement absents. L’implémentation ultérieure devra choisir et documenter location/connectedDevice et specialUse avec preuves de conformité et tests sur secteur/batterie.
