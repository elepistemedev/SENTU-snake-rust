# Phase 1: Invertir Dependencias y Romper el Ciclo versus.rs <-> viz_vs.rs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Romper la dependencia circular entre `versus.rs` y `viz_vs.rs` trasladando la responsabilidad de renderizado a la capa de presentación y haciendo de `versus.rs` un módulo 100% agnóstico a la UI.

**Architecture:** Inversión de dependencias (DIP) y separación de responsabilidades (SoC). `versus.rs` actúa exclusivamente como motor de partidas (dominio) y no conoce la UI. `viz_vs.rs` actúa como adaptador de renderizado (presentación) y consume el estado del match.

**Tech Stack:** Rust, Macroquad

**Spec:** [implementation_plan.md](file:///Users/el/.gemini/antigravity/brain/d834cecb-d75d-4819-9ec3-fca81a4253b0/implementation_plan.md)

## Global Constraints
- No alterar las fórmulas de puntuación, lógica de selección de ganador ni semántica de las series.
- Todos los 132 tests existentes deben seguir compilando y pasando.
- No introducir dependencias adicionales en Cargo.toml.

---

### Task 1: Desacoplar `src/versus.rs` de la Capa de Visualización

**Files:**
- Modify: `src/versus.rs`

**Interfaces:**
- Consumes: `crate::game::Game`, `crate::nn::Net`
- Produces: `VersusMatch::game1(&self) -> &Game`, `VersusMatch::game2(&self) -> &Game`, constructores `VersusMatch::new*` sin `flavor`

- [x] **Step 1: Modificar `VersusMatch` y constructores en `src/versus.rs`**
  - Remover `use crate::viz_vs::{VsFlavor, VizVS};`.
  - Remover `flavor: VsFlavor` del struct `VersusMatch`.
  - Actualizar `VersusMatch::new`, `VersusMatch::new_relative` y `VersusMatch::new_cross` para no aceptar `flavor`.
  - Añadir getters `pub fn game1(&self) -> &Game` y `pub fn game2(&self) -> &Game`.
  - Remover métodos `draw()` y `draw_with_series_info()` de `VersusMatch` y `BestOfSeries::draw()`.

- [x] **Step 2: Actualizar tests unitarios en `src/versus.rs`**
  - Remover función helper `inert_flavor()`.
  - Quitar el argumento `inert_flavor()` de todas las llamadas de test en `versus.rs`.

---

### Task 2: Implementar Helpers de Renderizado en `src/viz_vs.rs`

**Files:**
- Modify: `src/viz_vs.rs`

**Interfaces:**
- Consumes: `crate::versus::{VersusMatch, BestOfSeries, SeriesInfo}`
- Produces: `VizVS::draw_match(&self, match_: &VersusMatch, flavor: &VsFlavor)`, `VizVS::draw_series_match(&self, series: &BestOfSeries<B>, flavor: &VsFlavor)`

- [x] **Step 1: Importar tipos de `versus` y agregar métodos helper a `VizVS`**
  - Importar `VersusMatch` y `BestOfSeries`.
  - Añadir métodos `draw_match` y `draw_series_match` a `VizVS`.

---

### Task 3: Actualizar las Vistas (`view_ga_versus.rs`, `view_dqn_versus.rs`, `view_cross_match.rs`)

**Files:**
- Modify: `src/view_ga_versus.rs`
- Modify: `src/view_dqn_versus.rs`
- Modify: `src/view_cross_match.rs`

**Interfaces:**
- Consumes: `VizVS::draw_series_match`, `VersusMatch::new*` sin `flavor`
- Produces: Vistas renderizadas idénticas visualmente

- [x] **Step 1: Adaptar `view_ga_versus.rs`**
  - Almacenar `flavor: VsFlavor` en `GaVersusInner::Series`.
  - Llamar `VersusMatch::new(best, second_best)` en el closure.
  - En `draw()`, usar `VizVS::new().draw_series_match(series, flavor)`.

- [x] **Step 2: Adaptar `view_dqn_versus.rs`**
  - Almacenar `flavor: VsFlavor` en `DqnVersusInner::Series`.
  - Llamar `VersusMatch::new_relative(...)` en los closures sin pasar `flavor`.
  - En `draw()`, usar `VizVS::new().draw_series_match(series, flavor)`.

- [x] **Step 3: Adaptar `view_cross_match.rs`**
  - Almacenar `flavor: VsFlavor` en `CrossInner::Series`.
  - Llamar `VersusMatch::new_cross(ga, dqn)` sin `flavor`.
  - En `draw()`, usar `VizVS::new().draw_series_match(series, flavor)`.

---

### Task 4: Verificación Integral y Actualización del Grafo

- [x] **Step 1: Ejecutar la suite de tests (`cargo test`)**
  - Verificar que 132/132 tests pasan.
- [x] **Step 2: Verificar la ausencia de ciclos**
  - Comprobar que `src/versus.rs` no contiene `viz_vs`.
- [x] **Step 3: Ejecutar `graphify update .`**
