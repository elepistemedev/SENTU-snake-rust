# Especificación de Diseño: Renovación UI/UX Terminal Polish & Cohesive Cyber-Retro

- **Fecha**: 2026-09-11
- **Estado**: Aprobado por el usuario
- **Objetivo**: Elevar la interfaz visual y la experiencia de usuario (UI/UX) del proyecto Snake AI en Rust manteniendo su estética retro/terminal de arcade, logrando coherencia visual, responsividad en múltiples resoluciones y eliminando inconsistencias y duplicación de código.

---

## 1. Visión General y Principios de Diseño

1. **Estética Retro/Terminal Pulida**: Fondo oscuro profundo, bordes y esquinas de recuadro estilo terminal (`[ TITLE ]`), paleta de acentos de alta legibilidad (Cyan neón, Ámbar/Oro, Verde esmeralda, Coral/Rojo), con jerarquía tipográfica uniforme.
2. **Responsividad Dinámica**: Todo elemento visual se calcula proporcionalmente respecto a `screen_width()` y `screen_height()`, eliminando coordenadas verticales absolutas que provocaban solapamiento o recorte de gráficas en resoluciones inferiores a 1080p.
3. **Reutilización y DRY (Don't Repeat Yourself)**: Primitivas gráficas atómicas centralizadas en `src/ui_kit.rs` para estandarizar paneles, barras de progreso, gráficas y mensajes de estado.

---

## 2. Arquitectura de Componentes

```
src/
├── ui_kit.rs              [NUEVO] Primitivas visuales compartidas (paneles, badges, barras, charts)
├── theme.rs               Paletas de colores y persistencia de configuración
├── app.rs                 Menú principal renovado y vista de configuración de temas
├── dqn_dash.rs            Dashboard DQN responsivo con gráficas adaptables y red neuronal pulida
├── viz_advanced.rs        Dashboard GA responsivo alineado con la gramática visual unificada
├── viz_vs.rs              Arena versus con Series HUD corregido, marcador centrado y pips
├── view_dqn_versus.rs     Manejo de estados con vistas terminal limpias
├── view_ga_versus.rs      Manejo de estados con vistas terminal limpias
└── view_cross_match.rs    Manejo de estados con vistas terminal limpias
```

---

## 3. Especificación Detallada por Módulo

### 3.1 Primitivas Compartidas (`src/ui_kit.rs`)

Centraliza las funciones de dibujo y las constantes visuales del sistema:

* **Paleta base**:
  * `COLOR_BG`: `Color(0.04, 0.04, 0.06, 1.0)`
  * `PANEL_BG`: `Color(0.07, 0.08, 0.10, 0.95)`
  * `PANEL_BORDER`: `Color(0.25, 0.30, 0.38, 1.0)` (Grosor unificado: 2.0px)
  * `PANEL_BORDER_FOCUSED`: `Color(0.0, 0.85, 0.85, 1.0)`
  * `ACCENT_CYAN`: `Color(0.0, 0.90, 0.90, 1.0)`
  * `ACCENT_GOLD`: `Color(1.0, 0.80, 0.20, 1.0)`
  * `ACCENT_GREEN`: `Color(0.30, 0.90, 0.40, 1.0)`
  * `ACCENT_RED`: `Color(0.95, 0.30, 0.30, 1.0)`
  * `TEXT_MUTED`: `Color(0.60, 0.65, 0.70, 1.0)`
* **Funciones atómicas**:
  * `draw_terminal_box(x, y, w, h, title, focused)`: Dibuja un panel con esquinas rematadas, fondo semitransparente y título `[ TITLE ]`.
  * `draw_badge(text, x, y, badge_color)`: Etiqueta con fondo redondeado suave/recuadro para estados (`[REANUDABLE]`, `[ACTIVO]`, `[REQUIERE CHAMPION]`).
  * `draw_progress_bar(x, y, w, h, fraction, label, fill_color)`: Barra con borde delimitador y porcentaje exacto centrado.
  * `draw_responsive_chart(x, y, w, h, title, max_label, data, max_cap, bar_color)`: Gráfica de barras que distribuye dinámicamente el ancho de los slots y escala la altura al valor máximo local sin deformarse.
  * `draw_centered_text(text, cx, y, font_size, color)`: Helper universal con `measure_text` exacto.
  * `draw_missing_champion_notice(w, h, title, detail)`: Cuadro central de aviso cuando falta un modelo entrenado, con instrucciones claras y atajo de salida.

### 3.2 Menú Principal y Temas (`src/app.rs`)

* **Menú Principal**:
  * Caja de título terminal `SNAKE AI` con borde doble y subtítulo de control de agentes autónomos.
  * Filas interactivas con número de acceso directo `[ 1 ]` a `[ 6 ]`.
  * Badges de estado dinámicos en cada opción:
    * `DqnTrain`: Muestra `[PAUSADO - EP. X]` si hay entrenamiento activo, o `[CHAMPION LISTO]` si existe `dqn_champion.json`.
    * `DqnVersus`: Muestra `[REQUIERE CHAMPION]` en color atenuado si no hay modelo guardado.
    * `GaTrain`: Muestra `[PAUSADO - GEN. X]` si hay simulación activa.
    * `GaVersus` / `DqnVsGa`: Indicadores de estado de los archivos de campeones.
    * `ThemeConfig`: Muestra muestra de color del tema activo.
  * Selector activo: Borde cyan iluminado `PANEL_BORDER_FOCUSED` y flechas retro `▶  ◀`.
  * Barra de estado inferior estilo terminal de ancho completo con atajos y estado del sistema.
* **Configuración de Temas**:
  * Tarjetas de temas en columna izquierda con selector iluminado y badge `[ACTIVO]`.
  * Vista previa ("LIVE PREVIEW") en columna derecha: serpiente animada en bucle recorriendo una cuadrícula demostrando la continuidad de segmentos, ojos direccionales y onda de digestión.

### 3.3 Dashboards de Entrenamiento (`src/dqn_dash.rs` y `src/viz_advanced.rs`)

* **Layout Vertical Proporcional**:
  * Cálculo dinámico de altura para paneles de estadísticas superiores y barras de puntuación.
  * Espacio vertical restante asignado equitativamente a las dos gráficas de historial (`TIMES` y `SCORES`), garantizando que se rendericen completas en pantallas de cualquier resolución.
* **Normalización de Barras de Progreso**:
  * Eliminación del divisor fijo `/20.0` en GA.
  * Cálculo seguro de porcentaje relativo al récord de la sesión y al límite de pasos.
* **Visualización de Red Neuronal**:
  * Líneas de conexión con atenuación suave para evitar saturación de pantalla.
  * Resaltado visual en el nodo de acción seleccionada (argmax) con halo y etiqueta en `ACCENT_CYAN`.
  * Etiquetas de valores de salida formateadas con alineación limpia.
* **Panel de Hiperparámetros y Controles**:
  * Distribución tabular de parámetros en la esquina inferior izquierda.
  * Controles formateados con etiquetas de tecla: `[TAB] HUD   [R] Nuevo Agente   [ESC] Menú`.

### 3.4 Modos Versus y Partidas Cruzadas (`src/viz_vs.rs` y vistas asociadas)

* **Corrección del Series HUD**:
  * Ajuste de márgenes verticales para que el banner `🏆 [ JUGADOR ] GANA LA SERIE!` tenga espacio limpio sin solapar el borde del panel central.
* **Marcador Central Simétrico**:
  * Títulos y puntuaciones grandes centrados mediante cálculo de texto.
  * Pips circulares de victoria en la serie (`Best of 5`): círculos rellenos en el color del jugador para partidas ganadas, anillos tenues para partidas pendientes.
  * Tabla comparativa cara a cara (`SCORE`, `STEPS`, `FITNESS`).
* **Estados de Error / Modelos Faltantes**:
  * Reemplazo de la lógica duplicada en las 3 vistas versus por la llamada uniforme a `ui_kit::draw_missing_champion_notice`.

---

## 4. Plan de Verificación

1. **Compilación y Tests Unitarios**:
   - `cargo test` para asegurar que las pruebas headless de transición de modos y lógica pura continúen pasando al 100%.
2. **Pruebas Visuales en Resoluciones Múltiples**:
   - Ejecutar la aplicación en modo ventana/fullscreen y validar que los paneles no se recorten ni solapen.
   - Probar todas las vistas (Menú, DQN Train con y sin dashboard, GA Train, DQN Versus, GA Versus, Cross Match, Configuración de Temas).
3. **Validación de Temas**:
   - Cambiar entre los 4 temas en la pantalla de configuración y comprobar que la paleta se refleje de inmediato en el juego y en las vistas de entrenamiento.
