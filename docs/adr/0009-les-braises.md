# ADR 0009 — Refonte jouable v0.3

Le retour joueur rejette le rendu rétro, le démarrage et les commandes de la v0.2. La v0.3 adopte une présentation Android native, des illustrations originales et une entrée directe dans un prologue jouable. La simulation entière, le stockage et les systèmes DEV M0–M7 restent dans Rust.

La boucle principale devient une expédition : trois balises à rallumer, renforts, gardien final, trois vies, bilan et progression. Trois plans d’obstacles et trois comportements ennemis rendent le placement utile. L’assistance de visée est activée par défaut et peut être désactivée ; une visée manuelle reste disponible avec le pouce droit.

Le rendu suit le rafraîchissement Android ; une boucle distincte avance la simulation à 30 Hz. Les positions sont interpolées uniquement à l’affichage. Un menu de pause ne bloque pas les pairs d’un combat réseau. L’activité et le Phare conservent un seul propriétaire de la boucle à la fois.

Les sauvegardes antérieures ajoutent deux préférences par défaut sans changer le Nom. Le paquet et le certificat Android restent identiques et le versionCode passe à 3. Les salons de combat utilisent un espace v03 distinct, car les cartes et l’IA changent le résultat déterministe. Les dossiers v0.2 restent des preuves historiques ; leur rejeu nécessite la simulation v0.2.
