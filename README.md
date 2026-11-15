<p align="center">
  <img src="assets/app_icon.png" alt="REPUB Logo" width="110" />
</p>

<h1 align="center">REPUB</h1>

<p align="center">
  Lector de libros EPUB moderno, rápido y nativo construido en <strong>Rust + GPUI</strong>, con citas resaltadas, estadísticas de lectura y un asistente de IA con <strong>RAG temporal anti-spoilers</strong>.
</p>

---

## Características principales

- **Biblioteca local en SQLite:** Importación nativa de archivos `.epub` con extracción de metadatos, portada y tabla de contenidos. Búsqueda en tiempo real, ordenamiento y filtros (*Todos, Leyendo, Por leer, Terminados, Favoritos*).
- **Lector inmersivo y personalizable:**
  - Temas integrados (Dark, Light, Tokyo Night, Catppuccin, Nord, Dracula, Solarized, Glass) con transparencia y radio de bordes configurables.
  - Tipografía ajustable (familia serif/sans/mono, tamaño, interlineado, márgenes, ancho máximo de lectura y zoom).
- **Citas, resaltados y notas (`Ctrl+T` / `Ctrl+D`):**
  - Selección directa de texto sobre los párrafos con menú flotante.
  - Guardado de citas con 4 colores y notas personales persistidas en SQLite.
  - Marcadores de página y tabla de contenidos en panel lateral.
- **Asistente de lectura IA (RAG Temporal Anti-Spoilers):**
  - Soporta **OpenAI**, **Anthropic**, **Google Gemini** y **Groq**.
  - **Barrera estricta anti-spoilers:** solo recupera fragmentos y resúmenes hasta la página actual (`page_start <= current_page`).
  - **Recuperación híbrida y cronológica:** combina similitud semántica (embeddings) + búsqueda léxica (SQLite FTS5) + muestreo estratificado por capítulos y resúmenes progresivos bajo demanda.
  - **Embeddings configurables:** elegí entre modelos **locales** (`fastembed`, descarga bajo demanda desde Ajustes) o **cloud** (OpenAI / Gemini), con opción de deshabilitar la IA por libro desde la barra superior.
- **Inicio y estadísticas:** Tarjeta de *Continuar leyendo* con portada real, racha diaria, páginas leídas y tiempo de lectura.

---

## Atajos de teclado

| Atajo | Acción |
| :--- | :--- |
| `←` / `→` / `Espacio` | Página anterior / siguiente |
| `Ctrl + +` / `Ctrl + -` / `Ctrl + 0` | Zoom del lector |
| `Ctrl + T` | Abrir/cerrar panel lateral (Índice, Marcadores y Citas) |
| `Ctrl + D` | Guardar/quitar marcador en la página actual |
| `Ctrl + F` | Buscar en la biblioteca |
| `Ctrl + B` | Mostrar/ocultar barra lateral principal |
| `Esc` | Volver desde el lector |

---

## Descarga (Releases)

Podés descargar los ejecutables compilados directamente desde la pestaña **[Releases](../../releases)** de GitHub:

- **Windows (`x86_64`):** descargá `repub-vX.Y.Z-windows-x86_64.zip`, descomprimí y ejecutá `repub.exe`.
- **Linux (`x86_64`):**
  - **AppImage:** `chmod +x repub-vX.Y.Z-x86_64.AppImage && ./repub-vX.Y.Z-x86_64.AppImage`
  - **Portable (`.tar.gz`):** descomprimí y ejecutá `./repub`.

---

## Compilar desde el código fuente

### Requisitos previos
- **Rust** (toolchain estable)
- En **Linux (Ubuntu/Debian)**, librerías de sistema para GPUI y diálogos nativos:
  ```bash
  sudo apt-get install -y pkg-config libssl-dev libxkbcommon-dev libxkbcommon-x11-dev \
    libwayland-dev libvulkan-dev libgtk-3-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxcb-render0-dev libasound2-dev libfontconfig1-dev
  ```

### Ejecutar y testear
```bash
cargo check --all
cargo test --all
cargo run --bin repub
```
