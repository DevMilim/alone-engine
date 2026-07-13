# Relatório: Alone Engine — Prontidão para Game Jam

Análise do zip completo (`src/`, `macros/`, `example/`). Foco: o que já está pronto, o que vai travar você durante uma jam, e o que priorizar antes de começar.

## Resumo executivo

A engine está **acima da média** pra uma jam — a maioria dos motores caseiros nesse estágio não tem física com broad-phase espacial, integração de editor de nível (LDtk) e um sistema de rede cliente/servidor funcionando. Isso é trabalho pesado que normalmente consome metade do tempo de uma jam, e aqui já está feito.

O maior risco não é "falta funcionalidade" — é um **bug de protocolo de rede** que quebra silenciosamente toda comunicação cliente→servidor, e a **ausência total de renderização de texto/UI**, que praticamente todo jogo de jam precisa (menu, pontuação, "Game Over").

## Tabela de prioridades

| # | Item | Severidade | Esforço estimado |
|---|------|------------|-------------------|
| 1 | Bug: servidor não desserializa eventos do cliente | 🔴 Crítico (se o jogo for multiplayer) | Baixo — 1 função |
| 2 | Sem renderização de texto / `UiSystem` vazio | 🟠 Alto | Médio |
| 3 | `AudioSys::new()` panica sem dispositivo de áudio | 🟠 Alto | Baixo |
| 4 | Sprites não rotacionam visualmente | 🟡 Médio | Médio/Alto (dependendo do estilo do jogo) |
| 5 | Sem limitador de FPS explícito no loop | 🟡 Médio | Baixo |
| 6 | Campos mortos em `EventManager` (`server_mailbox`, `global_server_events`) | 🟡 Médio | Baixo |
| 7 | Carregamento de asset é síncrono (trava o frame) | 🟢 Baixo | Baixo (se só carregar no `start()`) |
| 8 | `Cargo.lock` no `.gitignore` do binário do jogo | 🟢 Baixo | Trivial |
| 9 | Typos (`iddle`, `PayerScene`) | 🟢 Cosmético | Trivial |

---

## O que já está pronto (pontos fortes)

- **Loop de jogo correto**: `WorldState::update` usa acumulador de tempo fixo (`FIXED_DT = 1/60`) com `blending` pra interpolação visual — é o padrão certo (fixed timestep + render interpolado), muita gente erra isso até em jams maiores.
- **Física 2D com broad-phase espacial**: `CollisionWorld` usa grid de células (`cell_of`, 64px) em vez de força bruta O(n²), com `move_and_slide` estilo Godot, `snap_to_floor`, colisores sensor (triggers), camadas/máscaras e **plataformas de mão única** (one-way). Isso sozinho já economiza dias de trabalho numa jam de plataforma.
- **Integração LDtk completa**: `Tilemap::from_ldtk_file` importa tiles, auto-layers e faz merge de colisores a partir do IntGrid (inclusive com merge horizontal de células idênticas). Você pode desenhar a fase no editor do LDtk em vez de hardcodar posições — grande economia de tempo.
- **Macros ergonômicas**: `#[derive(GameObject)]` com `#[base]/#[component]/#[object]` e `#[game(subscribe(...)/connect(...)/server_subscribe(...)/server_connect(...))]` reduzem MUITO boilerplate de dispatch e roteamento de eventos.
- **Rede cliente/servidor via WebSocket + bincode**, já assíncrono com `tokio`, com um padrão pub/sub pronto pros handlers (`server_subscribe`/`server_connect`) — só precisa do bug abaixo corrigido.
- **Áudio com mixer** (`rodio::Player`/`Mixer`), permitindo várias fontes simultâneas, loop e volume por instância.
- **Animação de sprite** com fps configurável e frame-rate independente (usa `delta`, não contagem de frames).
- **Matemática completa**: `Vector2` com `lerp`, `rotate`, `move_towards`, `clamp_magnitude`; `Transform` com hierarquia pai/filho (posição/rotação/escala globais).

---

## Bugs encontrados

### 1. 🔴 Servidor não desserializa mensagens do cliente

**Arquivo:** `src/network/server.rs`

Quando o servidor recebe um `Message::Binary` de um cliente conectado:

```rust
Ok(Message::Binary(data)) => {
    let server_event = ServerEvent::Broadcast(data.to_vec());
    // ...
}
```

Isso pega os bytes crus recebidos — que já são um `ServerEvent` inteiro serializado em bincode pelo cliente (`client.rs` serializa o enum inteiro antes de mandar pelo WebSocket) — e os embrulha *sem desserializar* dentro de um **novo** `ServerEvent::Broadcast`. Compare com o lado do cliente, que faz certo:

```rust
// client.rs — correto
Ok(Message::Binary(data)) => {
    if let Some(deserialized_event) = deserialize_bytes::<ServerEvent>(&data) {
        // ...
    }
}
```

**Impacto:**
- Qualquer `ctx.send_to_server(...)` ou `ctx.emit_to_server(...)` chega no servidor com o payload errado (bytes do enum inteiro, não do evento `E` original). Quando um handler `server_subscribe`/`server_connect` gerado pelo macro chama `server_event.event.decode::<T>()`, o bincode vai rejeitar (falha silenciosa, retorna `None`) ou, pior, decodificar algo sem sentido.
- Mensagens `ServerEvent::Targeted(id, ...)` do cliente **nunca são reconhecidas como tal** no servidor — tudo chega marcado como `Broadcast`, então `server_connect` (que casa em `ServerEvent::Targeted`) nunca dispara.
- Não é uma falha que gera pânico — é silenciosa. Debugar isso no meio de uma jam, sob pressão de tempo, é o pior cenário possível.
- Há inclusive um rastro no código: o macro (`macros/src/lib.rs`) tem um `println!("[RASTREIO 3] Macro tentou ler o Broadcast, mas o bincode rejeitou o tipo!")` só em debug — indício de que esse sintoma já apareceu antes.

**Correção sugerida** (espelhar o que `client.rs` já faz certo):

```rust
Ok(Message::Binary(data)) => {
    if let Some(server_event) = deserialize_bytes::<ServerEvent>(&data) {
        if tx_from_net_clone.send((peer_addr, server_event)).await.is_err() {
            break;
        }
    }
}
```

Se o jogo da jam **não for multiplayer**, isso não te afeta agora — mas corrija antes de confiar em `server_subscribe`/`server_connect` pra qualquer coisa.

### 2. 🟠 `UiSystem` é uma struct vazia — sem renderização de texto

```rust
// src/ui/mod.rs
pub struct UiSystem {}
```

`DrawCommand` só tem duas variantes: `Sprite` e `Rect`. Não existe glyph/bitmap-font rendering em nenhum lugar do `render/mod.rs`, apesar de haver fontes (`PixelOperator8.ttf`) nos assets de exemplo — elas não são usadas por nenhum código.

**Impacto:** placar, "Game Over", menus, diálogo, tutorial na tela — nada disso tem um caminho pronto. Você teria que:
- desenhar dígitos/letras manualmente como sprites de uma spritesheet (viável rápido, mas feio de configurar), ou
- escrever um rasterizador de bitmap font do zero durante a jam (arriscado sob prazo).

**Recomendação:** resolver isso **antes** da jam. Nem precisa ser bonito — um bitmap font 8x8 simples (tipo a fonte PixelOperator8 que já está nos assets) mapeado num spritesheet e uma função `draw_text(text, position, z_index)` que emite vários `DrawCommand::Sprite` já resolve 90% dos casos de jam.

### 3. 🟠 `AudioSys::new()` panica sem dispositivo de áudio

```rust
// src/audio.rs
let mut sink = DeviceSinkBuilder::open_default_sink().unwrap();
```

Isso roda dentro de `CoreSystems::default()`, chamado sempre que um `App` é criado — não tem como pular. Se rodar num ambiente sem placa de som/dispositivo de áudio disponível (comum em VPS headless, containers Docker, CI, ou o notebook de algum jurado com driver de áudio quebrado), a aplicação **crasha na inicialização**, antes até de abrir a janela.

Isso é especialmente relevante se algum dia vocês quiserem rodar um servidor dedicado headless (como discutido antes) — o `CoreSystems` do servidor também tentaria inicializar áudio e travaria.

**Recomendação:** trocar o `.unwrap()` por um fallback (log de aviso + um `AudioSys` "mudo" que ignora chamadas de play) em vez de derrubar o processo inteiro.

### 4. 🟡 Sprites não rotacionam visualmente

`Transform` tem `rotation`/`global_rotation` calculados corretamente em `apply_parent`, mas:
- `DrawCommand::Sprite` não carrega nenhum campo de ângulo;
- o blit em `render/mod.rs` só faz `flip_h`/`flip_v`, nunca rotação.

Ou seja: girar um `GameObject` (ex: `base.set_rotation(...)`) afeta a física/matemática, mas **visualmente o sprite não gira**. Se o jogo da jam for algo como twin-stick shooter, canhão giratório, carro que vira, etc., isso precisa ser adicionado (rotação de pixel a pixel numa engine puramente 2D-blit como essa não é trivial de fazer performática — vale planejar com antecedência, não descobrir isso no dia 2 da jam).

### 5. 🟡 Sem limitador de FPS / vsync explícito

`about_to_wait` roda a cada `ControlFlow::Poll` do winit sem nenhum `sleep`/limitador manual no código. O quanto isso importa depende do `PresentMode` padrão que a crate `pixels`/`wgpu` está usando por baixo dos panos — não deu pra confirmar isso sem rodar num display real. Vale testar em hardware de verdade antes da jam e, se a CPU/GPU ficar em 100% à toa, adicionar um limitador simples (dormir até o próximo frame alvo).

### 6. 🟡 Campos mortos em `EventManager`

```rust
pub global_server_events: VecDeque<ServerEvent>,
pub server_mailbox: IndexMap<Id, Vec<Box<dyn Any>>>,
```

Declarados e inicializados, mas nunca lidos nem escritos em lugar nenhum do código. Parece API planejada e abandonada no meio do caminho. Não atrapalha nada tecnicamente (só gera warning de "campo não lido"), mas confunde quem for ler o código depois — decida se termina ou remove.

---

## Riscos menores / observações de robustez

- **Carregamento de asset é síncrono**: `ImageAsset::load_from_file` chama `image::open` direto na thread principal, dentro do próprio frame que chamou `ctx.load_texture(...)`. Tranquilo se você só carregar no `start()` de uma cena, mas carregar um asset grande no meio do gameplay vai travar um frame perceptivelmente.
- **`.expect()`/`.unwrap()` em carregamento de asset**: um `path` errado de textura ou áudio derruba o processo inteiro (`ImageAsset::load_from_file` usa `.expect("Falha ao carregar textura")`). Comum de acontecer digitando caminho errado sob pressão de prazo — considere pelo menos uma mensagem de erro que diga qual path falhou (já tem a mensagem, mas sem o path — vale adicionar).
- **`Timer` usa `Instant::now()` (relógio de parede)**, não o tempo de simulação do jogo. Não tem conceito de "game time" pausável/escalável — se implementarem uma tela de pausa, os timers dos GameObjects continuam contando no fundo. Não é bug, é uma limitação de design a ter em mente.
- **Dependências de sistema pro build (Linux)**: `rodio` precisa de `libasound2-dev` (ALSA) e `winit` precisa de libs de X11/Wayland (`libxkbcommon-dev`, etc.) instaladas no SO. Se algum integrante da equipe for configurar uma máquina nova durante a jam, vale ter esse passo documentado no README pra não perder tempo.

---

## Funcionalidades que provavelmente vão faltar durante a jam (não são bugs, são gaps de escopo)

Pensando em jams típicas de 48–72h, provavelmente vocês vão sentir falta de:

- **Texto/UI** (já coberto acima — prioridade #1 de escopo)
- **Sistema de partículas** (comum pra "juice": explosões, poeira, rastro) — nada disso existe hoje
- **Save/load em disco** (highscore, progresso) — não há nenhuma API pra isso
- **Tela de loading com progresso** — como o carregamento de asset é síncrono e não há texto, dar feedback visual de "carregando..." é difícil hoje
- **Fullscreen/toggle de janela** — não vi nenhum atalho pra isso no `App`, só o resize já tratado

Nenhum desses é obrigatório pra uma jam (muitos jogos de jam saem sem save ou partículas), mas se o conceito do jogo depender de algum, é bom já ter um plano.

---

## Polish / nitpicks (zero impacto funcional)

- `"PayerScene"` em `example/src/main.rs` (provavelmente queria dizer `"PlayerScene"`).
- `iddle` em vez de `idle` (`create_iddle_animation`, nome de animação `"iddle"`) — espalhado pelo exemplo, sem problema técnico, só se quiserem revisar antes de mostrar código pros jurados.
- `.gitignore` do projeto ignora `Cargo.lock` — é a prática certa pra uma **biblioteca** (`alone-engine`), mas pro **binário do jogo** que vocês vão submeter na jam, o ideal é *commitar* o `Cargo.lock` pra garantir que o build dos jurados usa exatamente as mesmas versões de dependência que vocês testaram.

---

## Checklist recomendado antes da jam começar

1. [ ] Corrigir o bug de rede client→server em `network/server.rs` (se o jogo for multiplayer)
2. [ ] Implementar `draw_text` básico (bitmap font simples) e ligar num `UiSystem` real
3. [ ] Tornar `AudioSys::new()` resiliente a ausência de dispositivo de áudio (não travar o app)
4. [ ] Decidir se sprites precisam rotacionar visualmente — implementar se sim
5. [ ] Testar o loop principal numa máquina real e confirmar se precisa de limitador de FPS manual
6. [ ] Fazer um "smoke test" de onboarding: clonar o repo do zero numa máquina limpa, rodar `cargo run --release` no `example`, cronometrar quanto tempo/quantos passos manuais leva (documentar dependências de sistema que faltarem)
7. [ ] Decidir escopo de UI/partículas/save conforme o conceito do jogo escolhido na jam

Se quiser, posso te ajudar a implementar o `draw_text` (bitmap font) ou a correção do bug de rede primeiro — são os dois itens que mais valem a pena resolver antes do cronômetro da jam começar a rodar.
