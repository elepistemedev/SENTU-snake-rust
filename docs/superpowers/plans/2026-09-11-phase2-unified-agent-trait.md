# Phase 2: Abstracción del Trait Unificado Agent Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Crear el trait unificado `Agent` y las implementaciones `GaAgent` y `DqnPolicyAgent`, desacoplando `Game` y `VersusMatch` de las familias de redes y eliminando la bandera booleana `brain_relative`.

**Architecture:** Patrón Strategy / Inversión de Dependencias (DIP) y Polimorfismo mediante Traits en Rust. `Game` y `VersusMatch` interactúan con `Box<dyn Agent>`.

**Tech Stack:** Rust

**Spec:** [implementation_plan.md](file:///Users/el/.gemini/antigravity/brain/d834cecb-d75d-4819-9ec3-fca81a4253b0/implementation_plan.md)

## Global Constraints
- Mantener 100% de retrocompatibilidad con las pruebas unitarias y constructores existentes.
- Sin dependencias externas adicionales.

---

### Task 1: Crear el Módulo `src/agent.rs` con Trait `Agent` e Implementaciones

**Files:**
- Create: `src/agent.rs`
- Modify: `src/lib.rs`

**Interfaces:**
- Consumes: `crate::game::Game`, `crate::nn::Net`, `crate::utils::{relative_dir, rotate_vision_to_relative, FourDirs}`
- Produces: `trait Agent`, `GaAgent`, `DqnPolicyAgent`

- [x] **Step 1: Escribir el nuevo archivo `src/agent.rs` con trait, structs y tests unitarios**
- [x] **Step 2: Registrar `pub mod agent;` y `pub use agent::*;` en `src/lib.rs`**
- [x] **Step 3: Ejecutar `cargo test agent::tests` para verificar los nuevos tests**

---

### Task 2: Refactorizar `Game` para Usar `Option<Box<dyn Agent>>`

**Files:**
- Modify: `src/game.rs`

**Interfaces:**
- Consumes: `crate::agent::{Agent, GaAgent, DqnPolicyAgent}`
- Produces: `Game::with_agent(agent: Box<dyn Agent>) -> Self`, eliminación de `brain_relative`

- [x] **Step 1: Hacer público `get_four_dir_vision` en `Game`**
- [x] **Step 2: Añadir `pub agent: Option<Box<dyn Agent>>` y remover `brain_relative: bool`**
- [x] **Step 3: Actualizar `with_brain`, `with_relative_brain`, `get_brain_output` y `get_net_output`**
- [x] **Step 4: Ejecutar `cargo test game::tests` para confirmar que las pruebas de `Game` pasan**

---

### Task 3: Conectar `VersusMatch` con el Trait Polimórfico

**Files:**
- Modify: `src/versus.rs`

**Interfaces:**
- Consumes: `crate::agent::{Agent, GaAgent, DqnPolicyAgent}`
- Produces: `VersusMatch::from_agents(agent_left: Box<dyn Agent>, agent_right: Box<dyn Agent>) -> Self`

- [x] **Step 1: Agregar `VersusMatch::from_agents` y delegar los constructores existentes**
- [x] **Step 2: Agregar test de match polimórfico arbitrario**
- [x] **Step 3: Ejecutar `cargo test versus::tests`**

---

### Task 4: Verificación Integral y Actualización del Grafo de Graphify

- [x] **Step 1: Ejecutar la suite completa (`cargo test`)**
- [x] **Step 2: Ejecutar `graphify update . --code-only`**
