# engramme — sous-projet 1 : format de la base et skill de capture

- Date : 2026-10-20
- Statut : design validé en brainstorming, en attente de relecture de la spec
- Licence du dépôt : MIT, public dès le début

## 1. Contexte

engramme est une réimplémentation en salle blanche, plus moderne, de l'idée
d'AnkiMobile : révision espacée (FSRS), base de connaissances en Markdown et IA
tutrice. C'est une application desktop, pour un usage personnel.

Besoin d'origine : un analyste quantitatif risques travaille en permanence avec
un agent IA (Copilot au travail ; Claude Code, zcode et Codex en perso). L'agent
apprend en travaillant, mais l'humain n'en retient rien. engramme doit
capturer ce savoir au fil du travail, le soumettre à l'humain pour tri, puis le
faire réviser. Il doit aussi permettre de créer des cartes à la main, comme
d'habitude.

Contenus visés : détails de modélisation (équations, démonstrations, ordres de
grandeur), implémentation, spécificités des produits de garantie et de crédit,
ordres de grandeur de données.

### Décisions structurantes (valables pour tout le projet)

| Sujet | Décision | Raison |
|---|---|---|
| Forme | App desktop native (Tauri) | Choix de l'utilisateur. Risque : exécutable non signé bloqué sur Windows verrouillé → l'interface web doit aussi pouvoir être servie en local dans un navigateur (repli). |
| Stack | Rust (cœur) + Tauri + React/TypeScript (interface) | `fsrs-rs` est l'implémentation de référence ; binaire léger sans runtime ; empaquetage Python trop fragile sous Windows. |
| Source de vérité du contenu | Fichiers Markdown dans un dossier (la « base ») | Lisibles et modifiables par l'humain et par tous les agents ; versionnables avec git ; base du knowledge management. |
| Source de vérité de l'historique | Journal JSONL en ajout seul, versionné | Diffs lisibles ; l'état FSRS se recalcule en rejouant le journal ; l'optimisation FSRS en a besoin. |
| SQLite | Cache reconstructible, ignoré par git | Aucun binaire précieux à protéger. |
| Bases | Une base pro (poste Windows) et une base perso (Mac), **sans synchronisation** | Aucune fuite de contenu professionnel. |
| iPhone | Base perso seulement ; révision hors ligne indispensable | La base pro ne quitte pas le poste pro. Le format admet plusieurs appareils qui écrivent dans la même base (journal par appareil, identifiants aléatoires). |
| Dépôt | L'outil seul, jamais de données | Le dépôt est public. |
| Capture par les agents | Boîte de réception, tri humain obligatoire | Effet de génération : le tri est un moment d'apprentissage ; il évite que la pile de révisions grossisse sans fin. |
| IA | Facultative, fournisseurs interchangeables (Anthropic, OpenAI/Azure, Ollama) | L'app reste entièrement utilisable sans clé. Le LLM autorisé au travail n'est pas encore connu. |
| Notation de la réponse libre | L'IA commente, l'humain note | Un LLM qui corrige une démonstration se trompe ou est trop indulgent ; il ne doit pas fausser FSRS. |

### Feuille de route (sous-projets, dans l'ordre)

1. **Format + skill de capture** — ce document.
2. **App MVP** — révision Q/R et texte à trous avec FSRS, tri de la boîte de réception, consultation des notes. Inclut le repli navigateur, qui donne un premier accès iPhone : Mac allumé, Tailscale et PWA, en ligne seulement.
3. **iPhone hors ligne** — Tauri 2 pour iOS (même cœur Rust, même interface), synchronisation de la base perso (git ou iCloud Drive, à trancher), révision d'abord ; le tri et l'édition sur iPhone viendront ensuite. Placé avant l'IA : l'habitude de révision quotidienne compte plus que le tuteur.
4. **Couche IA** — fournisseurs, réponse libre commentée, explications, variantes d'exercices.
5. **Cartes d'ordre de grandeur et statistiques.**
6. **Knowledge management avancé** — liens `[[...]]`, rétroliens, recherche, graphe.

Chaque sous-projet a son propre cycle spec → plan → implémentation.

### Vérification préalable au sous-projet 2

Avant de concevoir l'app, vérifier sur le poste pro qu'un exécutable Tauri non
signé téléchargé depuis des releases GitHub se lance (installeur NSIS par
utilisateur). Le résultat détermine si le repli navigateur devient le mode
principal au travail.

## 2. Objectif du sous-projet 1

Fixer le contrat de données (format v1) et rendre la capture opérationnelle
**avant que l'app existe**. Pendant ce temps, l'utilisateur trie `inbox/` à la
main dans un éditeur. Ce tri manuel sert à valider le skill et à orienter le
design de l'app MVP.

Hors périmètre : interface, FSRS, IA tutrice, écriture d'identifiants dans les
notes, export ou import Anki.

## 3. Format v1

### 3.1 Structure de la base

```
<base>/                       # chemin donné par la variable ENGRAMME_DIR
  notes/                      # une note = un sujet ; sous-dossiers libres
    risque-credit/
      lgd-downturn.md
  inbox/                      # propositions des agents, en attente de tri
    2026-10-20T1432-copilot-ifrs9-staging.md
  .app/
    log/2026-10.mac.jsonl     # versionné ; un fichier par mois (UTC) et par appareil ; ajout seul
    log/2026-10.iphone.jsonl
    cache.sqlite              # ignoré par git ; reconstruit si absent
  .gitignore                  # contient au minimum .app/cache.sqlite
```

Fichiers en UTF-8 avec fins de ligne LF. L'analyseur accepte aussi CRLF, pour
les fichiers créés sous Windows.

### 3.2 Note

Une note est un fichier `.md` placé sous `notes/`. Elle commence par un
frontmatter YAML facultatif :

| Clé | Type | Obligatoire | Sens |
|---|---|---|---|
| `title` | chaîne | non (par défaut : le premier titre `#`, sinon le nom du fichier) | Titre affiché |
| `tags` | liste de chaînes | non | Classement, filtres de révision |
| `sources` | liste de chaînes | non | Origine du savoir (chemin, référence, URL) |

Les clés inconnues sont conservées et ignorées. Le reste de la note est du
Markdown libre (CommonMark + GFM), avec du LaTeX entre `$…$` et `$$…$$`.

### 3.3 Blocs de cartes

Une carte est un bloc délimité par une ligne d'ouverture `:::<type>` et une
ligne de fermeture `:::`. Les deux lignes commencent en colonne 0.

```
:::<type> {#<id>}
…contenu…
:::
```

- `<type>` ∈ `qa`, `cloze` (implémentés) ; `free`, `estimate` (réservés : un
  analyseur v1 les reconnaît, émet un avertissement et ne crée pas de carte).
- `{#<id>}` est facultatif. Forme : `c-` suivi de 6 à 8 caractères
  `[0-9a-z]`, **tirés au hasard** et jamais séquentiels, pour que deux
  appareils puissent attribuer des identifiants sans se coordonner. **Seule l'app attribue les identifiants** (sous-projet 2). Les
  agents et le skill n'en écrivent jamais. Une carte sans identifiant est
  valide.
- Un bloc ne peut pas contenir d'autre bloc. Une ligne `:::` à l'intérieur
  d'un bloc de code délimité (```) n'est pas une fermeture.

**`qa`** : recto, puis une ligne contenant exactement `---`, puis verso. La
première ligne `---` située hors d'un bloc de code sert de séparateur. Les deux
faces sont du Markdown, ne peuvent pas être vides, et peuvent s'étendre sur
plusieurs lignes, avec équations et code.

**`cloze`** : un seul corps Markdown contenant des trous `{{cN::texte}}` ou
`{{cN::texte::indice}}`, avec N ≥ 1 entier. Chaque N distinct produit une
sous-carte, identifiée `<id>.<N>` (ex. `c-91be0q.2`). Tous les trous portant le
même N sont masqués ensemble. Au moins un trou est obligatoire et le texte d'un
trou ne peut pas être vide. Les trous peuvent se trouver dans du LaTeX
(`$\mathrm{EL} = {{c1::PD}} \times \mathrm{LGD}$`) ; leur rendu est l'affaire
de l'app.

**Identité d'une carte.** L'historique est rattaché à l'identifiant, pas au
texte. Modifier le contenu d'une carte conserve son historique. Supprimer le
bloc retire la carte de la révision ; son historique reste dans le journal.

### 3.4 Fichier de la boîte de réception

Un fichier par proposition, placé dans `inbox/`. Nom :
`<AAAA-MM-JJ>T<HHMM>-<agent>-<sujet-court>.md`. Si le nom existe déjà, on
ajoute un suffixe `-2`, `-3`, etc.

Frontmatter **obligatoire** :

| Clé | Type | Sens |
|---|---|---|
| `proposed_by` | chaîne | `copilot`, `claude`, `codex`, `zcode`, … |
| `date` | date-heure ISO 8601 | Moment de la proposition |
| `context` | chaîne | Tâche en cours, en une phrase |
| `target` | chemin relatif à la base, sous `notes/` | Note existante à compléter, ou nouvelle note à créer |
| `why` | chaîne | Pourquoi ce savoir mérite d'être retenu |

Frontmatter facultatif : `tags` et `sources` (même sens que pour une note) ;
ils sont fusionnés dans la note cible à l'acceptation.

Corps : de la prose facultative (à ajouter à la note) et **de 1 à 3** blocs de
cartes, sans identifiant.

Le tri (sous-projet 2) se fait carte par carte : accepter, modifier puis
accepter, ou rejeter avec un motif parmi `trivial`, `faux`, `doublon`,
`trop-lie-au-code`, `autre`. Avant l'app, le tri se fait à la main : on copie
dans la note ce qu'on garde, puis on supprime le fichier.

### 3.5 Journal

`.app/log/<AAAA-MM>.<appareil>.jsonl` : une ligne JSON par événement,
horodatée en UTC. Le fichier du mois est choisi d'après `ts`. On n'y fait que
des ajouts, aucune réécriture.

`<appareil>` (`[a-z0-9-]+`, ex. `mac`, `iphone`, `poste-pro`) est configuré
localement sur chaque appareil, **hors de la base** (la base est synchronisée,
pas ce réglage). Chaque appareil n'écrit que dans ses propres fichiers. Deux
appareils ne modifient donc jamais le même fichier, et une synchronisation par
git ou iCloud ne crée aucun conflit sur le journal.

Champs communs : `ts` (ISO 8601 UTC), `ev` (type d'événement).

| `ev` | Champs supplémentaires | Producteur |
|---|---|---|
| `review` | `card` (id ou id.N), `rating` (1 à 4 : à revoir, difficile, bien, facile), `ms` (durée de réponse) | app (sous-projet 2) |
| `suspend` / `unsuspend` | `card` | app |
| `triage` | `inbox_file`, `action` (`accept`, `edit`, `reject`), `reason` (si rejet), `proposed_by` | app |

Le sous-projet 1 spécifie ce format ; il ne produit aucun événement. Le
cache SQLite se reconstruit en rejouant tous les fichiers du journal, tous
appareils confondus, dans l'ordre de `ts`. En cas d'égalité, on départage par
nom d'appareil, puis par ordre des lignes.

## 4. Skill de capture

Un seul fichier `skill/SKILL.md` (format Agent Skills : frontmatter `name` et
`description`, puis instructions), installable globalement dans chaque agent.
Les chemins exacts d'installation pour Claude Code, Codex et Copilot sont
vérifiés pendant l'implémentation et documentés dans `skill/INSTALL.md`.

### 4.1 Règles du skill

1. **Quand.** À la fin d'une tâche qui a mobilisé un savoir non trivial, pas à
   chaque échange. Au maximum **3 cartes par session**. Si rien ne mérite
   d'être retenu, ne rien proposer : c'est une issue normale.
2. **Quoi.** Ce que l'utilisateur devrait savoir *sans l'IA* : un choix de
   modélisation et sa justification, une équation, un ordre de grandeur, une
   spécificité de produit, la cause profonde d'un bug, un piège
   d'implémentation. **Pas** : chemins de fichiers, détails d'API qu'on
   retrouve en quelques secondes, informations éphémères.
3. **Lire avant d'écrire.** Chercher dans `notes/` (titres, tags, contenu) pour
   éviter les doublons et viser une note existante dans `target`.
4. **Bien écrire les cartes.** Une seule idée par carte ; une question
   compréhensible sans le code sous les yeux ; une réponse courte ; LaTeX pour
   les équations.
5. **Ce qui est interdit.** Secrets, identifiants de connexion, données
   personnelles, noms de clients ou de contreparties, valeurs de données
   brutes (arrondir les ordres de grandeur).
6. **Où écrire.** Uniquement dans `$ENGRAMME_DIR/inbox/`. Ne jamais modifier
   `notes/` ni `.app/`. Ne jamais écrire d'identifiant `{#…}`. Si
   `ENGRAMME_DIR` n'est pas défini ou n'existe pas, ne rien écrire et le
   signaler en une ligne.
7. **Après l'écriture.** Si la commande `engramme` est disponible, lancer
   `engramme check <fichier>` et corriger le fichier en cas d'erreur.
   Annoncer en une ligne ce qui a été proposé.

### 4.2 Fiabilité du déclenchement

Un skill est chargé à la discrétion de l'agent, qui risque de l'oublier.
Mesures :

- une ligne dans les instructions globales de chaque agent (CLAUDE.md,
  AGENTS.md, instructions utilisateur de Copilot) : « En fin de tâche,
  envisage le skill engramme-capture. » ;
- en option pour Claude Code, un hook `Stop` qui rappelle le skill ;
- un taux de déclenchement suivi pendant les deux premières semaines (nombre
  de fichiers dans `inbox/` par jour travaillé).

## 5. Crate `engramme-format` et CLI `check`

Un workspace Cargo :

- `crates/engramme-format` (bibliothèque) : analyse du frontmatter, des notes,
  des blocs et des fichiers de la boîte de réception ; modèle de données
  (`Note`, `Card { id: Option<CardId>, kind, … }`, `ClozeDeletion`,
  `InboxProposal`) ; diagnostics avec fichier, ligne, niveau (erreur ou
  avertissement) et message. Aucune E/S autre que la lecture de fichiers. Ce
  crate deviendra le cœur de l'app.
- `crates/engramme-cli` (binaire `engramme`) : la sous-commande
  `engramme check [<chemin>…]` valide une base entière (`ENGRAMME_DIR` ou le
  chemin donné) ou des fichiers isolés. Code de sortie 0 s'il n'y a aucune
  erreur, 1 sinon. Les diagnostics sont lisibles par un humain et par un agent.

### Diagnostics de `check`

Erreurs : bloc non fermé ; type de bloc inconnu ; face vide dans un `qa` ;
séparateur `---` absent dans un `qa` ; texte à trous sans trou, ou trou vide ;
identifiant mal formé ; identifiant en double dans la base ; frontmatter YAML
invalide ; clé obligatoire manquante dans un fichier de la boîte de
réception ; `target` hors de `notes/` ; plus de 3 cartes dans un fichier de la
boîte de réception ; identifiant présent dans un fichier de la boîte de
réception.

Avertissements : type réservé (`free`, `estimate`) ; numéros de trous non
consécutifs (ex. c1 et c3 sans c2).

## 6. Livrables

1. `docs/format.md` — la spec du format v1 destinée aux utilisateurs (section 3
   de ce document, avec des exemples).
2. `skill/SKILL.md` et `skill/INSTALL.md`.
3. Le workspace Cargo (`engramme-format`, `engramme-cli`).
4. `examples/savoir/` — une base d'exemple valide (quelques notes de risque de
   crédit génériques, un fichier dans la boîte de réception), qui sert aussi de
   jeu de test.
5. `README.md`, `LICENSE` (MIT).

## 7. Tests et critères de succès

- **Analyseur** : tests unitaires par règle de la section 3 ; fixtures valides
  et invalides sous `crates/engramme-format/tests/fixtures/` ; tests de
  référence (fichier Markdown → liste de cartes attendue). Chaque diagnostic
  de la section 5 a au moins un test qui le déclenche.
- **CLI** : `engramme check examples/savoir` sort en code 0 ; sur une fixture
  invalide, le code est 1 et le diagnostic indique le bon fichier et la bonne
  ligne.
- **Skill** : il n'existe pas de test automatique crédible. Critères à l'usage,
  après deux semaines de travail réel : des propositions arrivent ; 100 %
  passent `check` ; le taux de rejet, mesuré au tri manuel, est noté et
  sert à réviser le skill avant le sous-projet 2.

## 8. Questions ouvertes (sans effet sur le sous-projet 1)

- Synchronisation de la base perso entre Mac et iPhone : git ou iCloud Drive (sous-projet 3). Les notes Markdown restent exposées aux conflits si on les modifie sur les deux appareils ; les copies de conflit d'iCloud (« note 2.md ») devront être détectées.

- LLM autorisé au travail (à vérifier auprès de l'employeur).
- Lancement d'un exécutable Tauri non signé sur le poste pro (voir §1).
- Rendu du texte à trous à l'intérieur du LaTeX (sous-projet 2).
