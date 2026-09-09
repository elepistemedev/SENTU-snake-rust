# Deep Q-Network (DQN) Implementation

## 🎯 Diferencias con Algoritmo Genético

### Algoritmo Genético (rama `main`)
- **Población**: 1000 serpientes compiten simultáneamente
- **Aprendizaje**: Evolución generacional mediante selección natural
- **Método**: Crossover y mutación de pesos
- **Sin memoria**: Cada serpiente nace y muere sin aprender individualmente

### DQN (rama `feature/dqn`)
- **Agente único**: Una serpiente aprende continuamente
- **Aprendizaje**: Refuerzo mediante Q-learning
- **Método**: Backpropagation con experience replay
- **Con memoria**: El agente aprende de experiencias pasadas

## 🧠 Componentes DQN

### 1. Q-Network
- Red neuronal que estima Q(s,a) para cada acción
- Arquitectura: 12 → 8 → 4 (igual que GA)
- Predice el valor esperado de cada acción

### 2. Target Network
- Copia de Q-network actualizada cada 100 pasos
- Estabiliza el entrenamiento
- Previene oscilaciones en los valores Q

### 3. Experience Replay Buffer
- Almacena últimas 10,000 experiencias (s, a, r, s', done)
- Entrena con batches aleatorios de 32 experiencias
- Rompe correlación temporal entre experiencias

### 4. Epsilon-Greedy
- **Exploración**: Acción aleatoria con probabilidad ε
- **Explotación**: Mejor acción según Q-network
- ε decae de 1.0 → 0.01 (decay: 0.995)

## 📊 Sistema de Recompensas

```rust
+1.0   → Come comida
+0.1   → Se acerca a la comida
-0.01  → Cada paso (penalización por tiempo)
-1.0   → Colisión (pared o cuerpo)
```

## 🚀 Uso

El crate compila **un único binario** (`snake`) que abre un menú con las cinco
vistas: entrenar DQN, versus DQN, entrenar GA, versus GA y DQN vs GA. La vista
de entrenamiento DQN (y el GA) se *pausa* al volver al menú — no se destruye —
y desde el menú se reanuda con la misma tecla.

### Ejecutar la app unificada:
```bash
cargo run --release
```

### Menú (pantalla inicial)

| Tecla | Acción |
| --- | --- |
| `1` | Entrenar DQN (crea agente nuevo la primera vez; reanuda la sesión pausada si existe) |
| `2` | Versus DQN (champion vs política actual; partida nueva en cada entrada) |
| `3` | Entrenar GA (algoritmo genético; DQN-style HUD, `Tab` panel avanzado) |
| `4` | Versus GA (best-ever vs second-best) |
| `5` | DQN vs GA (best_snake.json vs dqn_champion.json) |
| `Up`/`Down` + `Enter` | Navegar y seleccionar |
| `Esc` | Salir de la app |

El menú indica si hay una sesión DQN pausada reanudable, p. ej. `1) DQN Train
(paused at episode N - press 1 to resume)`.

### Teclas dentro de las vistas

| Vista | Teclas |
| --- | --- |
| DQN train | `R` = agente nuevo (conserva el champion); `Esc` = pausar y volver al menú |
| GA train | `Espacio` = lento/rápido; `Tab` = panel avanzado; `V` = versus interno GA; `Esc` = volver al menú |
| Versus / cross | `Esc` = volver al menú (también al terminar: `Esc`/`Enter`) |

El DQN **champion** (snapshot del q-network en cada récord de sesión) se guarda
en `dqn_champion.json` (formato serde `Net`, mismo que `best_snake.json`). El
archivo está en `.gitignore`; si falta o está corrupto la app arranca sin
champion y las vistas que lo necesitan muestran un mensaje (no crashea).

> Nota: el binario `snake-dqn` ya no existe — la app unificada reemplaza ambos
> binarios anteriores (`snake` y `snake-dqn`).

## 📈 Hiperparámetros

```rust
REPLAY_BUFFER_SIZE: 10,000
BATCH_SIZE: 32
GAMMA (γ): 0.99          // Factor de descuento
LEARNING_RATE: 0.001
EPSILON_START: 1.0
EPSILON_END: 0.01
EPSILON_DECAY: 0.995
TARGET_UPDATE: 100 pasos
```

## 🔄 Flujo de Entrenamiento

1. **Observar** estado actual (visión 4 direcciones)
2. **Seleccionar** acción (ε-greedy)
3. **Ejecutar** acción y recibir recompensa
4. **Almacenar** experiencia (s, a, r, s', done)
5. **Entrenar** con batch aleatorio del buffer
6. **Actualizar** target network cada 100 pasos
7. **Repetir** hasta convergencia

## 📝 Ecuación de Bellman

```
Q(s,a) = r + γ · max(Q(s',a'))
```

Donde:
- `Q(s,a)`: Valor de acción `a` en estado `s`
- `r`: Recompensa inmediata
- `γ`: Factor de descuento (0.99)
- `max(Q(s',a'))`: Mejor valor Q en siguiente estado

## 🎮 Controles

- `Esc` en el menú — Salir de la app
- `Esc` en una vista — Volver al menú (entrenamiento DQN/GA queda pausado y reanudable)
- `R` en DQN train — Reiniciar con un agente nuevo (conserva el champion)

## 📊 Métricas Mostradas

- **Episode**: Número de partida actual
- **Score**: Longitud de la serpiente en partida actual
- **Best**: Mejor score alcanzado
- **Epsilon**: Tasa de exploración actual

## 🔬 Ventajas de DQN vs GA

✅ **Aprendizaje continuo** - No reinicia desde cero cada generación
✅ **Eficiencia de datos** - Reutiliza experiencias pasadas
✅ **Convergencia más rápida** - En problemas con recompensas claras
✅ **Adaptabilidad** - Puede ajustarse a cambios en el entorno

## ⚠️ Limitaciones Actuales

- Backpropagation simplificado (aproximación)
- Sin optimizador Adam/RMSprop
- Red neuronal básica sin capas convolucionales
- Para producción, considerar usar librerías como `tch-rs` (PyTorch bindings)

## 🚧 Mejoras Futuras

- [ ] Double DQN (reduce sobreestimación)
- [ ] Dueling DQN (separa V(s) y A(s,a))
- [ ] Prioritized Experience Replay
- [ ] Optimizador Adam
- [x] Guardar/cargar modelos entrenados (champion snapshot en `dqn_champion.json`)

## 📁 Archivos generados

- `dqn_champion.json` — snapshot del q-network en cada récord de sesión DQN (cubierto por `.gitignore`)
- `best_snake.json` / `sim_metadata.json` — champions e historia del algoritmo genético
- [ ] Gráficas de progreso en tiempo real
