# 0003 — Sauvegarde chiffrée atomique pour l’alpha
Contexte : l’objectif immédiat est une partie Android jouable et récupérable hors ligne. Le journal contient déjà des événements signés, mais le stockage append-only et sa compaction sous Sceau ne sont pas implémentés.
Décision : snapshot JSON local chiffré AES-GCM dans AtomicFile, qui contient le registre et l’état. En erreur de lecture ou d’authentification, arrêter l’ouverture sans remplacer le fichier. Sauvegarder périodiquement et à la pause.
Alternatives rejetées : SharedPreferences en clair, remise à zéro silencieuse, cloud de sauvegarde.
Conséquences : adapté à une alpha locale et à de petits historiques ; pas au registre mondial canonique. Les 24 mots restaurent une identité, pas une histoire perdue sans autre copie.
