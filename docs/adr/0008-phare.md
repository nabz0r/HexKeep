# ADR 0008 — Veille Android explicite

Le Phare est démarré depuis l'application au premier plan et utilise un service visible de types `location|connectedDevice`, une notification persistante avec action Arrêter, des balises BLE et la position GPS locale. Il n'utilise pas de permission de localisation permanente ni de notification distante. La simulation reste locale ; aucun serveur d'autorité n'est ajouté.

Le service s'arrête s'il perd le secteur ou le Wi-Fi ou si l'ancre GPS se déplace de plus de 200 m. Sa boucle ne double pas celle de l'écran actif. Le système peut le tuer ; il ne redémarre pas silencieusement. La sauvegarde chiffrée préserve l'état.

Le mode DEV autorise les essais sans témoin physique. La qualification des appareils réels, les politiques d'énergie des fabricants et la consommation doivent être mesurées sur matériel. Aucune autonomie en heures n'est annoncée à partir d'un émulateur.

Les écritures de l'Activity et du service partagent un verrou de disque commun afin de protéger le même fichier atomique chiffré. Le service écrit seulement en arrière-plan, puis à son arrêt. Un Phare sauvegardé actif n'est pas présenté comme encore actif après la mort du processus : il doit être réactivé, et une double demande de démarrage ne crée pas deux horloges de simulation.
