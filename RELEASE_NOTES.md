## 🚀 O que há de novo na v0.5.0 / What's New in v0.5.0

### 🇧🇷 Português
- 🧩 **Libcosmic atualizado:** Compatibilidade com o commit mais recente do COSMIC.
- 💬 **Legendas mais eficientes:** Solicita português e inglês dos EUA sem variantes duplicadas.
- 🚦 **Progresso mais claro:** Mostra preparação, legendas e pós-processamento.
- 🧹 **Limpeza segura:** Remove apenas arquivos temporários pertencentes ao download.
- ⚠️ **Falhas parciais:** Limites HTTP 429 do YouTube não cancelam o vídeo.
- 🐱 **Identidade visual:** Mascote Dixycat e seção Sobre atualizados.

### 🇺🇸 English
- 🧩 **Updated libcosmic:** Compatibility with the latest COSMIC commit.
- 💬 **More efficient subtitles:** Requests Portuguese and US English without duplicate variants.
- 🚦 **Clearer progress:** Shows preparation, subtitles, and post-processing stages.
- 🧹 **Safe cleanup:** Removes only temporary files belonging to the current download.
- ⚠️ **Partial failures:** YouTube HTTP 429 limits no longer cancel the video.
- 🐱 **Visual identity:** Updated Dixycat mascot and About section.

---

## 🚀 O que há de novo na v0.4.3 / What's New in v0.4.3

### 🇧🇷 Português
- 🧩 **Libcosmic atualizado:** Compatibilidade com o commit mais recente do COSMIC.
- 💬 **Legendas mais eficientes:** Solicita português e inglês dos EUA sem variantes duplicadas.
- 🚦 **Progresso mais claro:** Mostra preparação, legendas e pós-processamento.
- 🧹 **Limpeza segura:** Remove apenas os arquivos `.vtt` do download atual.
- ⚠️ **Falhas parciais:** Limites HTTP 429 do YouTube não cancelam o vídeo.
- 🐱 **Identidade visual:** Adicionado o mascote Dixycat na seção Sobre.

### 🇺🇸 English
- 🧩 **Updated libcosmic:** Compatibility with the latest COSMIC commit.
- 💬 **More efficient subtitles:** Requests Portuguese and US English without duplicate variants.
- 🚦 **Clearer progress:** Shows preparation, subtitles, and post-processing stages.
- 🧹 **Safe cleanup:** Removes only `.vtt` files belonging to the current download.
- ⚠️ **Partial failures:** YouTube HTTP 429 limits no longer cancel the video.
- 🐱 **Visual identity:** Added the Dixycat mascot to the About section.

---

## 🚀 O que há de novo na v0.4.2 / What's New in v0.4.2

### 🇧🇷 Português
- ⚙️ **Runtime JavaScript incluído:** Os pacotes agora incluem Deno, usado pelo yt-dlp para lidar melhor com desafios JavaScript do YouTube.

---

### 🇺🇸 English
- ⚙️ **Bundled JavaScript runtime:** Packages now include Deno, used by yt-dlp to handle YouTube JavaScript challenges more reliably.

---

### 🇪🇸 Español
- ⚙️ **Runtime JavaScript incluido:** Los paquetes ahora incluyen Deno, utilizado por yt-dlp para gestionar de forma más fiable los desafíos JavaScript de YouTube.

---

## 🚀 O que há de novo na v0.4.1 / What's New in v0.4.1

### 🇧🇷 Português
- 💬 **Legendas mais confiáveis:** O applet baixa apenas legendas em português e inglês, com intervalo entre solicitações. Uma falha de legenda não cancela mais o vídeo.
- 📦 **Dependências do pacote:** Os pacotes `.deb` e `.rpm` agora declaram ffmpeg, Python, mutagen, Node.js e Polkit.
- 🔄 **Reinício após atualização:** O applet executa a nova versão automaticamente depois que a atualização é instalada.

---

### 🇺🇸 English
- 💬 **More reliable subtitles:** The applet downloads only Portuguese and English subtitles, with a delay between requests. A subtitle failure no longer cancels the video.
- 📦 **Package dependencies:** `.deb` and `.rpm` packages now declare ffmpeg, Python, mutagen, Node.js, and Polkit.
- 🔄 **Restart after updating:** The applet automatically runs the new version after an update is installed.

---

### 🇪🇸 Español
- 💬 **Subtítulos más confiables:** El applet descarga solo subtítulos en portugués e inglés, con una pausa entre solicitudes. Un error de subtítulos ya no cancela el vídeo.
- 📦 **Dependencias del paquete:** Los paquetes `.deb` y `.rpm` ahora declaran ffmpeg, Python, mutagen, Node.js y Polkit.
- 🔄 **Reinicio después de actualizar:** El applet ejecuta automáticamente la nueva versión después de instalar una actualización.

---

## 🚀 O que há de novo na v0.4.0 / What's New in v0.4.0

### 🇧🇷 Português
- 🛑 **Cancelamento confiável:** O botão de cancelar agora encerra a busca de título e o download em andamento.
- 🎵 **YouTube Music:** Melhor compatibilidade com links do YouTube Music.
- 🐛 **Diagnóstico:** Use `--debug`, `-d` ou `COSMIC_YTDLP_DEBUG=1` para gerar um log detalhado.

---

### 🇺🇸 English
- 🛑 **Reliable cancellation:** The cancel button now stops both title lookup and the active download.
- 🎵 **YouTube Music:** Improved compatibility with YouTube Music links.
- 🐛 **Diagnostics:** Use `--debug`, `-d`, or `COSMIC_YTDLP_DEBUG=1` to create a detailed log.

---

### 🇪🇸 Español
- 🛑 **Cancelación confiable:** El botón de cancelar ahora detiene la búsqueda del título y la descarga activa.
- 🎵 **YouTube Music:** Compatibilidad mejorada con enlaces de YouTube Music.
- 🐛 **Diagnóstico:** Usa `--debug`, `-d` o `COSMIC_YTDLP_DEBUG=1` para crear un registro detallado.

---

## 🚀 O que há de novo na v0.3.0 / What's New in v0.3.0

### 🇧🇷 Português
- 📝 **Seleção de Legendas:** Novo seletor de legendas com opções: "Não", "Sim" (legenda embutida no arquivo de vídeo) e "Sim, (Junto e separado em .vtt)" (legenda embutida + arquivo `.vtt` separado na pasta).
- 💬 **Filtro de Legendas:** Ignora automaticamente transmissões de live chat (`live_chat`) para evitar downloads indesejados de arquivos `.json`.
- 🔄 **Botão de Atualização com Feedback:** Novo botão na barra superior ao lado de plataformas ("!") com status dinâmico ("Verificando por atualizações...", "Há uma atualização!", "Sem atualizações") e link direto para instalação.
- 🎵 **Correção no Download de Áudio (Opus/FLAC/WAV):** Corrigido o erro onde downloads de áudio geravam arquivos `.webp` e `.png` órfãos e mostravam notificação de falha devido à ausência do módulo Python `mutagen`. Agora o applet detecta compatibilidade de capas com `ffmpeg` e limpa quaisquer resíduos temporários.

---

### 🇺🇸 English
- 📝 **Subtitle Mode Selector:** New subtitle option with "No", "Yes" (embedded in video), and "Yes (embedded + .vtt)" (embeds subtitle + keeps separate `.vtt` file in folder).
- 💬 **Subtitle Filter:** Automatically excludes `live_chat` transcripts to avoid cluttering folders with raw `.json` chat replays.
- 🔄 **Update Check Button with Visual Feedback:** Dedicated refresh button next to platforms ("!") with live status ("Checking for updates…", "Update available!", "No updates") and direct install support.
- 🎵 **Audio Download Bug Fix (Opus/FLAC/WAV):** Fixed an issue where audio downloads left behind orphaned `.webp`/`.png` thumbnail files and reported a false error notification when python `mutagen` was missing. Cover art embedding now checks ffmpeg compatibility and cleans up temporary files safely.

---

### 🇪🇸 Español
- 📝 **Selector de Subtítulos:** Nueva opción con "No", "Sí" (incrustado en el video) y "Sí (incrustado + .vtt)" (incrustado + archivo `.vtt` independiente).
- 💬 **Filtro de Subtítulos:** Excluye automáticamente `live_chat` para evitar descargas de archivos `.json` de chats en vivo.
- 🔄 **Botón de Actualización con Estados:** Botón al lado de plataformas ("!") con retroalimentación en tiempo real.
- 🎵 **Corrección en Descarga de Audio:** Se corrigió el error donde las descargas de audio dejaban archivos `.webp`/`.png` huérfanos y mostraban un aviso falso de error.

---

## 🚀 O que há de novo na v0.2.9 / What's New in v0.2.9

### 🇧🇷 Português
- 🌿 **Modo Econômico:** Novo toggle para economizar dados — baixa em resolução/bitrate menor para economizar banda e armazenamento.
- 🔄 **Verificação e Instalação Automática de Atualizações:** Botão "Verificar Atualizações" que consulta a API do GitHub Releases e instala automaticamente usando polkit (sem necessidade de terminal).
- 🔗 **Validação de URL:** O applet agora valida a URL antes de iniciar o download, evitando erros silenciosos com links inválidos.
- 🚦 **Limite de Banda:** Nova opção nas configurações para limitar a velocidade de download.
- ⚡ **Spinner durante extração de metadados:** Feedback visual enquanto o yt-dlp calcula duração/metadados (especialmente útil para Shorts).
- 🐛 **Correção da barra de progresso para YouTube Shorts:** Regressão corrigida — a barra agora avança corretamente em Shorts.
- ⚠️ **Erros de legenda não fatais:** Falhas ao baixar legendas não interrompem mais o download principal.
- ⏱️ **Timeouts de rede:** Adicionados timeouts para evitar travamentos em conexões lentas ou instáveis.

---

### 🇺🇸 English
- 🌿 **Economy Mode:** New toggle to save data — downloads at lower resolution/bitrate to reduce bandwidth and storage usage.
- 🔄 **Automatic Update Check & Install:** "Check for Updates" button queries GitHub Releases API and auto-installs the new version using polkit (no terminal needed).
- 🔗 **URL Validation:** The applet now validates the URL before starting a download, preventing silent failures with invalid links.
- 🚦 **Bandwidth Limit:** New settings option to cap download speed.
- ⚡ **Spinner during metadata extraction:** Visual feedback while yt-dlp calculates duration/metadata (especially helpful for Shorts).
- 🐛 **Progress bar fix for YouTube Shorts:** Regression fixed — progress bar now advances correctly on Shorts.
- ⚠️ **Non-fatal subtitle errors:** Subtitle download failures no longer abort the main download.
- ⏱️ **Network timeouts:** Added timeouts to prevent hanging on slow or unstable connections.

---

### 🇪🇸 Español
- 🌿 **Modo Económico:** Toggle para ahorrar datos — descarga en menor resolución/bitrate para reducir uso de banda y almacenamiento.
- 🔄 **Verificación e Instalación Automática de Actualizaciones:** Botón que consulta la API de GitHub Releases e instala automáticamente usando polkit.
- 🔗 **Validación de URL:** La aplicación valida la URL antes de iniciar la descarga.
- 🚦 **Límite de Ancho de Banda:** Nueva opción en configuraciones para limitar la velocidad de descarga.
- ⚡ **Spinner durante extracción de metadatos:** Retroalimentación visual mientras yt-dlp calcula metadatos.
- 🐛 **Corrección de barra de progreso en YouTube Shorts:** Regresión corregida.
- ⚠️ **Errores de subtítulos no fatales:** Los fallos al descargar subtítulos ya no abortan la descarga principal.
- ⏱️ **Timeouts de red:** Añadidos timeouts para evitar bloqueos en conexiones lentas.

---

## 🚀 O que há de novo na v0.2.7 / What's New in v0.2.7

### 🇧🇷 Português
- 🔄 **Auto-Atualização Automática do yt-dlp:** O applet agora verifica e baixa automaticamente a versão mais recente do `yt-dlp` em segundo plano ao iniciar, garantindo compatibilidade contínua com mudanças do YouTube sem necessidade de atualizar o applet manualmente.
- ⚠️ **Notificação de Atualização em Caso de Falha:** Se um download falhar devido a mudanças no YouTube ou erros do extrator, o applet tenta atualizar o `yt-dlp` automaticamente e notifica o usuário para tentar novamente.
- 🔍 **Detecção Proativa de Versões:** Verificação da versão instalada do `yt-dlp` e comparação com a versão estável mais recente nos releases oficiais do GitHub.

---

### 🇺🇸 English
- 🔄 **Automatic yt-dlp Self-Update:** The applet now automatically checks and downloads the latest `yt-dlp` version in the background on startup, ensuring continuous compatibility with YouTube changes without requiring manual applet updates.
- ⚠️ **Update Notification on Failure:** If a download fails due to YouTube changes or extractor errors, the applet attempts to auto-update `yt-dlp` and notifies the user to retry.
- 🔍 **Proactive Version Detection:** Checks installed `yt-dlp` version against the latest stable release from official GitHub releases.

---

### 🇪🇸 Español
- 🔄 **Auto-Actualización Automática de yt-dlp:** La aplicación ahora verifica y descarga automáticamente la última versión de `yt-dlp` en segundo plano al iniciar.
- ⚠️ **Notificación de Actualización en Caso de Fallo:** Si una descarga falla, la aplicación intenta actualizar `yt-dlp` automáticamente.
- 🔍 **Detección Proactiva de Versiones:** Verificación de la versión instalada de `yt-dlp`.
