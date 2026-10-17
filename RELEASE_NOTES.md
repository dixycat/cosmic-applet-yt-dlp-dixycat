## 🚀 O que há de novo na v0.6.5 / What's New in v0.6.5

### 🇧🇷 Português
- 📦 **Instalação no Pop!_OS corrigida:** O pacote `.deb` agora depende de `pkexec`, disponível nos repositórios atuais, em vez do pacote transitório `policykit-1`.
- 📊 **Progresso de download corrigido:** A interface permanece em preparação enquanto o yt-dlp busca metadados/legendas e entra na etapa de download ao detectar o arquivo ou receber progresso; também recupera a etapa correta após baixar legendas e capas.
- 🧩 **Dependências COSMIC atualizadas:** Atualizado o libcosmic e o conjunto de dependências travado no `Cargo.lock`.

### 🇺🇸 English
- 📦 **Pop!_OS installation fixed:** The `.deb` package now depends on `pkexec`, available in current repositories, instead of the transitional `policykit-1` package.
- 📊 **Download progress fixed:** The UI stays in preparation while yt-dlp fetches metadata/subtitles, switches to downloading when the file or transfer progress is detected, and recovers the correct stage after subtitles and thumbnails.
- 🧩 **Updated COSMIC dependencies:** Updated libcosmic and the dependency set pinned in `Cargo.lock`.
