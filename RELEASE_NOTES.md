## 🚀 O que há de novo na v0.6.6 / What's New in v0.6.6

### 🇧🇷 Português
- 🧩 **Dependências e COSMIC atualizados:** Sincronizado com o `libcosmic` mais recente e atualizadas as dependências do ecossistema COSMIC.
- ⚡ **Bibliotecas modernizadas:** `yt-dlp` atualizado para 2.10, `ashpd` atualizado para 0.14 (com suporte a `zbus` 6), `reqwest` para 0.13 e `i18n-embed` para 0.16.
- 📦 **Binário mais enxuto:** Eliminadas duplicatas de dependências transitivas no empacotamento.

### 🇺🇸 English
- 🧩 **Updated COSMIC and dependencies:** Synchronized with the latest upstream `libcosmic` and updated COSMIC ecosystem dependencies.
- ⚡ **Modernized libraries:** Upgraded `yt-dlp` to 2.10, `ashpd` to 0.14 (with `zbus` 6 support), `reqwest` to 0.13, and `i18n-embed` to 0.16.
- 📦 **Cleaner binary:** Eliminated duplicate transitive dependency versions in packaging.

---

## 🚀 O que há de novo na v0.6.5 / What's New in v0.6.5

### 🇧🇷 Português
- 📦 **Instalação no Pop!_OS corrigida:** O pacote `.deb` agora depende de `pkexec`, disponível nos repositórios atuais, em vez do pacote transitório `policykit-1`.
- 📊 **Progresso de download corrigido:** A interface permanece em preparação enquanto o yt-dlp busca metadados/legendas e entra na etapa de download ao detectar o arquivo ou receber progresso; também recupera a etapa correta após baixar legendas e capas.
- 🧩 **Dependências COSMIC atualizadas:** Atualizado o libcosmic e o conjunto de dependências travado no `Cargo.lock`.

### 🇺🇸 English
- 📦 **Pop!_OS installation fixed:** The `.deb` package now depends on `pkexec`, available in current repositories, instead of the transitional `policykit-1` package.
- 📊 **Download progress fixed:** The UI stays in preparation while yt-dlp fetches metadata/subtitles, switches to downloading when the file or transfer progress is detected, and recovers the correct stage after subtitles and thumbnails.
- 🧩 **Updated COSMIC dependencies:** Updated libcosmic and the dependency set pinned in `Cargo.lock`.
