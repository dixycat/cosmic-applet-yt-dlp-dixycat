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
