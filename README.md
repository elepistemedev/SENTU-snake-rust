# IA-juego-snake-rust

Proyecto experimental de IA para juego Snake con múltiples enfoques:

- **Snake DQN** — Deep Q-Network con PyTorch
- **Snake Trading** — Aplicación a trading algorítmico
- **Limit Order Book (LOB)** — Visualización de order book
- **Frontend** — UI de visualización
- **Damocles** — Agente de trading integrado

## Estructura

Los módulos viven en subdirectorios. Los artefactos entrenados, modelos y
caches se excluyen del control de versiones.

---

## Snake Rust — app unificada (crate `snake`)

El crate Rust (`src/`, binario `snake`) compila **un único binario** que unifica
los antiguos `snake` (genético) y `snake-dqn` (DQN) en un menú con cinco vistas.

### Ejecutar

```bash
cargo run --release
```

Ventana a pantalla completa (`snake-ai`). Menú: `1` DQN train, `2` DQN versus,
`3` GA train, `4` GA versus, `5` DQN vs GA — también navegable con
`Up`/`Down` + `Enter`. `Esc` en el menú sale de la app; `Esc` en una vista vuelve
al menú pausando el entrenamiento (reanudable desde el menú).

### Teclas por vista

- **DQN train**: `R` = agente nuevo (conserva el champion); `Esc` = menú.
- **GA train**: `Espacio` = lento/rápido; `Tab` = dashboard avanzado; `V` = versus interno GA; `Esc` = menú.
- **Versus / cross**: `Esc` (o `Enter` al terminar) = menú.

### Champions

- `dqn_champion.json` — snapshot del q-network en cada récord DQN (`.gitignore`;
  faltante/corrupto → sin champion, sin crash).
- `best_snake.json` / `sim_metadata.json` — champions del algoritmo genético
  (`.gitignore`).
