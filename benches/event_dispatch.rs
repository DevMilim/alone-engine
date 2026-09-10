use std::{
    any::{Any, TypeId},
    future::Future,
    hint::black_box,
    sync::Arc,
};

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use tokio::runtime::Handle;
use winit::{event::MouseButton, keyboard::KeyCode};

use alone_engine::{
    audio::AudioAsset,
    collision::{ColliderData, ColliderKey, CollisionFlag, Layer},
    core::{
        AssetApi, AudioApi, Base, CollisionApi, CoreApi, EngineApi, EventApi, GameObject,
        GameObjectBase, Handler, Id, InputApi, Pool, RenderApi, SceneApi, WorldApi,
    },
    event::{CallbackEvent, GlobalEvent},
    math::{Vector2, Vector2i},
    render::ImageAsset,
    runtime::{AsyncContext, GameObjectDispatch, Scene},
};

use rodio::Player;

// ============================================================
// Benchmark object
// ============================================================

#[derive(Clone, Copy)]
struct BenchMessage;

struct BenchObject {
    base: Base,
    received: u64,
}

impl BenchObject {
    fn new() -> Self {
        Self {
            base: Base::default(),
            received: 0,
        }
    }
}

impl GameObjectBase for BenchObject {
    fn base(&self) -> &Base {
        &self.base
    }

    fn base_mut(&mut self) -> &mut Base {
        &mut self.base
    }
}

impl GameObject for BenchObject {
    type Message = BenchMessage;

    #[inline(always)]
    fn on_message(&mut self, _ctx: &mut impl EngineApi, _msg: &Self::Message) {
        self.received += 1;
    }
}

impl GameObjectDispatch for BenchObject {
    fn dispatch_start(&mut self, _ctx: &mut impl EngineApi, _base: &Base) {}

    #[inline]
    fn dispatch_events(&mut self, ctx: &mut impl EngineApi) {
        if let Some(events) = ctx.take_mailbox(self.base.id) {
            for event in events {
                match event {
                    GlobalEvent::Send(_, any_event) | GlobalEvent::Targeted(_, any_event) => {
                        if let Some(message) = any_event.downcast_ref::<BenchMessage>() {
                            self.on_message(ctx, message);
                        }
                    }
                    // Broadcast NUNCA chega pela mailbox no modelo atual.
                    GlobalEvent::Broadcast(_) => {}
                }
            }
        }

        // Leitura por cursor — reproduz o broadcast_read_block gerado
        // pela macro pra quem tem #[subscribe(_: BenchMessage)].
        let type_id = TypeId::of::<BenchMessage>();
        let (start, end) = ctx.broadcast_range(self.base.id, type_id);
        for i in start..end {
            if let Some(any_event) = ctx.get_broadcast(type_id, i) {
                if let Some(message) = any_event.downcast_ref::<BenchMessage>() {
                    self.on_message(ctx, message);
                }
            }
        }
    }

    fn dispatch_update(&mut self, _ctx: &mut impl EngineApi, _base: &Base, _delta: f32) {}
    fn dispatch_late_update(&mut self, _ctx: &mut impl EngineApi, _base: &Base, _delta: f32) {}
    fn dispatch_fixed_update(&mut self, _ctx: &mut impl EngineApi, _base: &Base, _delta: f32) {}
    fn dispatch_draw(&mut self, _ctx: &mut impl RenderApi, _base: &Base, _blending: f32) {}
    fn dispatch_destroy(&mut self, _ctx: &mut impl EngineApi) {}
}

// ============================================================
// Minimal EngineApi mock — agora com broadcast log + cursors
// ============================================================

#[derive(Default)]
struct BenchContext {
    mailbox: FxHashMap<Id, Vec<GlobalEvent>>,
    broadcasts: FxHashMap<TypeId, Vec<Arc<dyn Any + Send + Sync>>>,
    broadcast_cursors: FxHashMap<(Id, TypeId), usize>,
}

// ... CoreApi, WorldApi, InputApi, AssetApi, AudioApi, CollisionApi,
// SceneApi continuam idênticos (todos unreachable!())

impl EventApi for BenchContext {
    fn send<T: Send + 'static>(&mut self, _id: Id, _message: T) {
        unreachable!()
    }
    fn send_boxed_any(&mut self, _id: Id, _message: Box<dyn Any + Send + 'static>) {
        unreachable!()
    }
    fn emit<T: Send + 'static>(&mut self, _event: T) {
        unreachable!()
    }
    fn emit_targeted<T: Send + 'static>(&mut self, _id: Id, _event: T) {
        unreachable!()
    }
    fn send_service<T: 'static, E: Send + 'static>(&mut self, _event: E) {
        unreachable!()
    }

    #[inline]
    fn take_mailbox(&mut self, id: Id) -> Option<Vec<GlobalEvent>> {
        self.mailbox.remove(&id)
    }

    fn register_subscriptions(&mut self, _id: Id, _types: &[TypeId]) {
        unreachable!()
    }
    fn unregister_subscriptions(&mut self, _id: Id, _types: &[TypeId]) {
        unreachable!()
    }

    #[inline]
    fn broadcast_range(&mut self, id: Id, type_id: TypeId) -> (usize, usize) {
        let len = self.broadcasts.get(&type_id).map_or(0, Vec::len);
        let cursor = self.broadcast_cursors.entry((id, type_id)).or_insert(0);
        let start = (*cursor).min(len);
        *cursor = len;
        (start, len)
    }

    #[inline]
    fn get_broadcast(&self, type_id: TypeId, index: usize) -> Option<Arc<dyn Any + Send + Sync>> {
        self.broadcasts.get(&type_id)?.get(index).cloned()
    }
}

// ... AssetApi/AudioApi/CollisionApi/SceneApi/EngineApi iguais

// ============================================================
// Benchmark: Broadcast — modelo atual (log compartilhado + cursor)
// ============================================================
fn benchmark_broadcast_dispatch(c: &mut Criterion) {
    let legacy_sizes = [1_000usize, 5_000, 10_000, 25_000, 50_000, 100_000];
    let engine_sizes = [
        1_000usize, 5_000, 10_000, 25_000, 50_000, 100_000, 250_000, 500_000,
    ];

    let mut group = c.benchmark_group("broadcast_dispatch");
    group.sample_size(30);

    for &object_count in &engine_sizes {
        let broadcast_count = object_count / 4;

        group.throughput(Throughput::Elements(broadcast_count as u64));

        // LEGACY (inalterado — continua fiel ao comportamento antigo,
        // que nunca teve por-instância subscription).
        if legacy_sizes.contains(&object_count) {
            group.bench_with_input(
                BenchmarkId::new("legacy_broadcast", object_count),
                &object_count,
                |b, &object_count| {
                    b.iter_batched(
                        || {
                            let objects = create_objects(object_count);
                            let events: Vec<LegacyBroadcastEvent> = (0..broadcast_count)
                                .map(|_| LegacyBroadcastEvent::Broadcast(Box::new(BenchMessage)))
                                .collect();
                            (objects, events, BenchContext::default())
                        },
                        |(mut objects, events, mut ctx)| {
                            for event in &events {
                                legacy_dispatch_broadcast(&mut objects, &mut ctx, event);
                            }
                            black_box(received_sum(&objects));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }

        // ENGINE — modelo atual: TODO objeto do pool é "assinante"
        // aqui só pra manter throughput comparável ao legacy (fan-out
        // total); inserção é O(1) por broadcast, independente disso.
        group.bench_with_input(
            BenchmarkId::new("engine_broadcast", object_count),
            &object_count,
            |b, &object_count| {
                b.iter_batched(
                    || {
                        let pool = create_objects(object_count);
                        let mut ctx = BenchContext::default();
                        let type_id = TypeId::of::<BenchMessage>();

                        // Inserção: 1 push por broadcast, SEM iterar
                        // assinantes — essa é a mudança central.
                        for _ in 0..broadcast_count {
                            let payload: Arc<dyn Any + Send + Sync> = Arc::new(BenchMessage);
                            ctx.broadcasts.entry(type_id).or_default().push(payload);
                        }

                        (pool, ctx)
                    },
                    |(mut pool, mut ctx)| {
                        GameObjectDispatch::dispatch_events(&mut pool, &mut ctx);
                        black_box(received_sum(&pool));
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================
// Dispatcher real da engine (modelo novo: mailbox pull-based)
// ============================================================
//
// Send, Targeted e Broadcast são tratados igual do ponto de vista do
// objeto: tudo chega pela mailbox de take_mailbox(id), O(1) por
// objeto. O que muda é SÓ o roteamento na inserção (fora daqui).

// ============================================================
// Minimal EngineApi mock
// ============================================================

impl CoreApi for BenchContext {
    fn camera_mut(&mut self) -> &mut Vector2 {
        unreachable!()
    }

    fn window_size(&self) -> (u32, u32) {
        unreachable!()
    }

    fn async_ctx(&self) -> AsyncContext {
        unreachable!()
    }

    fn async_task<F>(&mut self, _owner_id: Id, _future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        unreachable!()
    }

    fn blocking_task<F>(&mut self, _owner_id: Id, _future: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        unreachable!()
    }

    fn abort_tasks_of(&mut self, _id: Id) {
        unreachable!()
    }

    fn register_service<T: 'static>(&mut self, _id: Id) {
        unreachable!()
    }

    fn service_id<T: 'static>(&self) -> Option<Id> {
        unreachable!()
    }

    fn async_handle(&mut self) -> &mut Handle {
        unreachable!()
    }

    fn set_state<T: 'static>(&mut self, _value: T) {
        unreachable!()
    }

    fn get_state<T: 'static>(&self) -> Option<&T> {
        unreachable!()
    }

    fn get_state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        unreachable!()
    }

    fn remove_state<T: 'static>(&mut self) {
        unreachable!()
    }

    fn range(&mut self, _range: std::ops::Range<i32>) -> i32 {
        unreachable!()
    }
}

impl WorldApi for BenchContext {
    fn spawn<T: GameObject + Send + 'static>(&mut self, _obj: T) {
        unreachable!()
    }

    fn register_alive(&mut self, _id: Id) {
        unreachable!()
    }

    fn unregister_alive(&mut self, _id: Id) {
        unreachable!()
    }

    fn destroy(&mut self, _id: Id) {
        unreachable!()
    }
}

impl InputApi for BenchContext {
    fn is_key_pressed(&self, _key: KeyCode) -> bool {
        unreachable!()
    }

    fn is_key_just_pressed(&self, _key: KeyCode) -> bool {
        unreachable!()
    }

    fn mouse_position(&self) -> Vector2 {
        unreachable!()
    }

    fn is_mouse_pressed(&self, _key: MouseButton) -> bool {
        unreachable!()
    }

    fn is_mouse_just_pressed(&self, _key: MouseButton) -> bool {
        unreachable!()
    }

    fn is_action_pressed(&self, _action: &str) -> bool {
        unreachable!()
    }

    fn is_action_just_pressed(&self, _action: &str) -> bool {
        unreachable!()
    }

    fn get_vector(
        &self,
        _action_up: &str,
        _action_down: &str,
        _action_left: &str,
        _action_right: &str,
    ) -> Vector2 {
        unreachable!()
    }

    fn get_key_vector(
        &self,
        _key_up: KeyCode,
        _key_down: KeyCode,
        _key_left: KeyCode,
        _key_right: KeyCode,
    ) -> Vector2 {
        unreachable!()
    }

    fn get_key_axis(&self, _negative_key: KeyCode, _positive_key: KeyCode) -> f32 {
        unreachable!()
    }

    fn get_axis(&self, _negative_action: &str, _positive_action: &str) -> f32 {
        unreachable!()
    }
}

impl AssetApi for BenchContext {
    fn load_texture(&mut self, _owner: Id, _path: &str) -> Handler<ImageAsset> {
        unreachable!()
    }

    fn load_texture_and_resize(
        &mut self,
        _owner: Id,
        _path: &str,
        _width: u32,
        _height: u32,
    ) -> Handler<ImageAsset> {
        unreachable!()
    }

    fn load_audio(&mut self, _owner: Id, _path: &str) -> Handler<AudioAsset> {
        unreachable!()
    }

    fn unload_texture(&mut self, _owner: Id, _texture: Handler<ImageAsset>) {
        unreachable!()
    }

    fn unload_audio(&mut self, _owner: Id, _audio: Handler<AudioAsset>) {
        unreachable!()
    }

    fn clear_assets(&mut self) {
        unreachable!()
    }
}

impl AudioApi for BenchContext {
    fn play(&mut self, _sound: Handler<AudioAsset>, _looped: bool) -> Player {
        unreachable!()
    }
}

impl CollisionApi for BenchContext {
    fn update_collider_geometry(
        &mut self,
        _key: ColliderKey,
        _layer: Layer,
        _mask: Layer,
        _is_sensor: bool,
        _on_way_collision: bool,
        _size: (i32, i32),
        _fallback_pos: (i32, i32),
    ) {
        unreachable!()
    }

    fn update_collider(&mut self, _key: ColliderKey, _data: ColliderData) {
        unreachable!()
    }

    fn remove_collider(&mut self, _key: ColliderKey) {
        unreachable!()
    }

    fn move_and_slide(
        &mut self,
        _my_id: Id,
        _position: &mut Vector2,
        _velocity: &mut Vector2,
    ) -> CollisionFlag {
        unreachable!()
    }

    fn snap_to_floor(&mut self, _my_id: Id, _snap_length: i32) -> Option<i32> {
        unreachable!()
    }

    fn translate_my_colliders(&mut self, _my_id: Id, _offset: Vector2i) {
        unreachable!()
    }

    fn check_collisions(&mut self, _my_id: Id) -> bool {
        unreachable!()
    }

    fn resolve_axis(
        &mut self,
        _my_id: Id,
        _position: &mut Vector2i,
        _velocity: &mut Vector2i,
        _is_x_axis: bool,
    ) -> Vector2i {
        unreachable!()
    }

    fn register_trigger_callbacks(
        &mut self,
        _key: ColliderKey,
        _on_enter: Option<CallbackEvent>,
        _on_exit: Option<CallbackEvent>,
    ) {
        unreachable!()
    }
}

impl SceneApi for BenchContext {
    fn push_scene<T: Scene + 'static>(&mut self, _scene: T) {
        unreachable!()
    }

    fn change_scene<T: Scene + 'static>(&mut self, _scene: T) {
        unreachable!()
    }

    fn pop_scene(&mut self) {
        unreachable!()
    }

    fn clear_scene(&mut self) {
        unreachable!()
    }
}

impl EngineApi for BenchContext {}

// ============================================================
// Modelo antigo (pré-mailbox), reproduzido localmente
// ============================================================
//
// No EventManager antigo, Targeted/Broadcast viviam numa fila global
// (`global_events: VecDeque<GlobalEvent>`), e o driver fazia:
//   while let Some(event) = pop_front() { pool.dispatch_event(ctx, &event) }
// Ou seja: CADA evento disparava uma travessia completa da árvore, e
// cada objeto comparava seu próprio Id contra o Id do evento. Não há
// early-exit possível nesse desenho — a árvore não sabe "onde" está
// o alvo, então todo nó paga o custo de ser visitado, mesmo quando
// não é ele o destinatário.
enum LegacyEvent {
    Targeted(Id, Box<dyn Any>),
}

fn legacy_dispatch_targeted(
    objects: &mut Pool<BenchObject>,
    ctx: &mut BenchContext,
    event: &LegacyEvent,
) {
    let LegacyEvent::Targeted(id, any_event) = event;

    for object in objects.iter_mut() {
        if &object.base().id == id {
            if let Some(message) = any_event.downcast_ref::<BenchMessage>() {
                object.on_message(ctx, message);
            }
        }
    }
}

// ============================================================
// Modelo antigo de Broadcast
// ============================================================
//
// #[subscribe(handler: EventType)] era resolvido em TEMPO DE
// COMPILAÇÃO, por tipo — não existia registro por instância (Id).
// Ou seja: toda instância do tipo processava TODO broadcast daquele
// tipo, sem exceção — não havia como um objeto específico dizer
// "não quero ouvir isso agora". Diferente do Targeted (onde o custo
// extra era só a comparação de Id), aqui o custo extra é o próprio
// handler sendo chamado em objetos que, na versão nova, nem estariam
// inscritos.
enum LegacyBroadcastEvent {
    Broadcast(Box<dyn Any>),
}

fn legacy_dispatch_broadcast(
    objects: &mut Pool<BenchObject>,
    ctx: &mut BenchContext,
    event: &LegacyBroadcastEvent,
) {
    let LegacyBroadcastEvent::Broadcast(any_event) = event;

    for object in objects.iter_mut() {
        // Fidelidade ao modelo antigo: TODO objeto do tipo processa,
        // não só uma fração "inscrita" (esse conceito não existia
        // por instância).
        if let Some(message) = any_event.downcast_ref::<BenchMessage>() {
            object.on_message(ctx, message);
        }
    }
}

// ============================================================
// Old mailbox (usado só no grupo Send / manual_baseline)
// ============================================================

type OldMailbox = IndexMap<Id, Vec<Box<dyn Any>>, FxBuildHasher>;

fn manual_dispatch_message(
    objects: &mut Pool<BenchObject>,
    mailbox: &mut OldMailbox,
    ctx: &mut BenchContext,
) {
    for object in objects.iter_mut() {
        if mailbox.is_empty() {
            return;
        }

        if let Some(messages) = mailbox.remove(&object.base().id) {
            for message in messages {
                if let Some(message) = message.downcast_ref::<BenchMessage>() {
                    object.on_message(ctx, message);
                }
            }
        }
    }
}

// ============================================================
// Helpers
// ============================================================

fn create_objects(count: usize) -> Pool<BenchObject> {
    let mut pool = Pool::default();

    for _ in 0..count {
        pool.spawn(BenchObject::new());
    }

    pool
}

fn target_ids(pool: &Pool<BenchObject>, message_count: usize) -> Vec<Id> {
    (0..message_count)
        .map(|i| {
            let position = i * pool.iter().count() / message_count;

            pool.iter().nth(position).unwrap().base().id
        })
        .collect()
}

fn received_sum(pool: &Pool<BenchObject>) -> u64 {
    pool.iter().map(|object| object.received).sum()
}

// ============================================================
// Benchmark: Send (mailbox já existia no old EventManager)
// ============================================================

fn benchmark_send_dispatch(c: &mut Criterion) {
    let sizes = [
        1_000usize, 5_000, 10_000, 25_000, 50_000, 100_000, 250_000, 500_000,
    ];

    let mut group = c.benchmark_group("send_dispatch");
    group.sample_size(30);

    for &object_count in &sizes {
        let message_count = object_count / 4;

        let template = create_objects(object_count);
        let targets = target_ids(&template, message_count);
        drop(template);

        group.throughput(Throughput::Elements(message_count as u64));

        group.bench_with_input(
            BenchmarkId::new("manual_baseline", object_count),
            &object_count,
            |b, &object_count| {
                b.iter_batched(
                    || {
                        let objects = create_objects(object_count);
                        let mut mailbox: OldMailbox =
                            IndexMap::with_hasher(FxBuildHasher::default());

                        for &target in &targets {
                            mailbox.insert(target, vec![Box::new(BenchMessage)]);
                        }

                        (objects, mailbox, BenchContext::default())
                    },
                    |(mut objects, mut mailbox, mut ctx)| {
                        manual_dispatch_message(&mut objects, &mut mailbox, &mut ctx);
                        black_box(received_sum(&objects));
                    },
                    BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("engine_dispatch_events", object_count),
            &object_count,
            |b, &object_count| {
                b.iter_batched(
                    || {
                        let pool = create_objects(object_count);
                        let mut ctx = BenchContext::default();

                        for &target in &targets {
                            ctx.mailbox
                                .entry(target)
                                .or_default()
                                .push(GlobalEvent::Send(target, Box::new(BenchMessage)));
                        }

                        (pool, ctx)
                    },
                    |(mut pool, mut ctx)| {
                        GameObjectDispatch::dispatch_events(&mut pool, &mut ctx);
                        black_box(received_sum(&pool));
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================
// Benchmark: Targeted (aqui mora o problema que o old NÃO resolvia)
// ============================================================

fn benchmark_targeted_dispatch(c: &mut Criterion) {
    // legacy explode exponencialmente; acima de 100k o tempo de coleta
    // fica impraticável (250k já passa de 25 minutos) e não agrega
    // informação nova além do que 1k-100k já comprova.
    let legacy_sizes = [1_000usize, 5_000, 10_000, 25_000, 50_000, 100_000];
    let engine_sizes = [
        1_000usize, 5_000, 10_000, 25_000, 50_000, 100_000, 250_000, 500_000,
    ];

    let mut group = c.benchmark_group("targeted_dispatch");
    group.sample_size(30);

    for &object_count in &engine_sizes {
        let message_count = object_count / 4;

        let template = create_objects(object_count);
        let targets = target_ids(&template, message_count);
        drop(template);

        group.throughput(Throughput::Elements(message_count as u64));

        if legacy_sizes.contains(&object_count) {
            group.bench_with_input(
                BenchmarkId::new("legacy_targeted", object_count),
                &object_count,
                |b, &object_count| {
                    b.iter_batched(
                        || {
                            let objects = create_objects(object_count);
                            let events: Vec<LegacyEvent> = targets
                                .iter()
                                .map(|&target| {
                                    LegacyEvent::Targeted(target, Box::new(BenchMessage))
                                })
                                .collect();
                            (objects, events, BenchContext::default())
                        },
                        |(mut objects, events, mut ctx)| {
                            for event in &events {
                                legacy_dispatch_targeted(&mut objects, &mut ctx, event);
                            }
                            black_box(received_sum(&objects));
                        },
                        BatchSize::SmallInput,
                    );
                },
            );
        }

        group.bench_with_input(
            BenchmarkId::new("engine_targeted", object_count),
            &object_count,
            |b, &object_count| {
                b.iter_batched(
                    || {
                        let pool = create_objects(object_count);
                        let mut ctx = BenchContext::default();
                        for &target in &targets {
                            ctx.mailbox
                                .entry(target)
                                .or_default()
                                .push(GlobalEvent::Targeted(target, Box::new(BenchMessage)));
                        }
                        (pool, ctx)
                    },
                    |(mut pool, mut ctx)| {
                        GameObjectDispatch::dispatch_events(&mut pool, &mut ctx);
                        black_box(received_sum(&pool));
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// ============================================================
// Benchmark: Broadcast (fan-out para N assinantes, não 1 alvo)
// ============================================================
//
// Diferença chave vs Targeted: um Targeted tem exatamente 1
// destinatário; um Broadcast tem potencialmente MUITOS assinantes.
// No legacy, "assinante" era decidido por TIPO em tempo de
// compilação — então aqui simulamos o pior caso real: TODO objeto
// do pool processa TODO broadcast. No engine, só quem está
// registrado (via register_subscriptions) recebe, então usamos uma
// fração do pool (`subscribers`) como assinantes de verdade.

criterion_group!(
    benches,
    benchmark_send_dispatch,
    benchmark_targeted_dispatch,
    benchmark_broadcast_dispatch
);

criterion_main!(benches);
