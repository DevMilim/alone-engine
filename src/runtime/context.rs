use crate::{
    audio::AudioAsset,
    collision::Layer,
    core::{AssetApi, AudioApi, CoreApi, Handler, InputApi, SceneApi, TriggerCallbacks, WorldApi},
    math::Vector2i,
    rng::Random,
    runtime::{AppCommands, State},
};
use std::{
    any::{Any, TypeId},
    sync::{Arc, mpsc::Sender},
};

use rodio::Player;
use winit::keyboard::KeyCode;

use crate::{
    collision::{ColliderData, ColliderKey, CollisionFlag},
    core::{CollisionApi, CoreSystems, EngineApi, EventApi, GameObject, Id},
    event::{EventManager, GlobalEvent, SpawnEvent},
    math::Vector2,
    render::ImageAsset,
};

pub struct EngineContext<'a> {
    pub systems: &'a mut CoreSystems,
    pub events: &'a mut EventManager,
    pub camera_position: &'a mut Vector2,
    pub state: &'a mut State,
    pub window_size: &'a (u32, u32),
    pub is_fixed_update: bool,
    pub rng: &'a mut Random,
}

impl<'a> EngineApi for EngineContext<'a> {}
impl<'a> CoreApi for EngineContext<'a> {
    fn camera_mut(&mut self) -> &mut Vector2 {
        self.camera_position
    }

    fn window_size(&self) -> (u32, u32) {
        *self.window_size
    }

    fn async_ctx(&self) -> AsyncContext {
        AsyncContext {
            sender: self.systems.bg_event_sender.clone(),
        }
    }

    fn async_task<F>(&mut self, owner_id: Id, future: F)
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let handle = self.systems.async_handle.spawn(future);
        let handles = self.systems.task_handles.entry(owner_id).or_default();
        handles.retain(|h| !h.is_finished());
        handles.push(handle);
    }
    fn blocking_task<F>(&mut self, owner_id: Id, task: F)
    where
        F: FnOnce() -> () + Send + 'static,
    {
        let handle = self.systems.async_handle.spawn_blocking(task);
        let handles = self.systems.task_handles.entry(owner_id).or_default();
        handles.retain(|h| !h.is_finished());
        handles.push(handle);
    }
    fn abort_tasks_of(&mut self, id: Id) {
        if let Some(handles) = self.systems.task_handles.remove(&id) {
            for handle in handles {
                handle.abort();
            }
        };
    }

    fn register_service<T: 'static>(&mut self, id: Id) {
        let type_id = TypeId::of::<T>();
        self.systems.service_register.insert(type_id, id);
    }

    fn service_id<T: 'static>(&self) -> Option<Id> {
        self.systems
            .service_register
            .get(&TypeId::of::<T>())
            .copied()
    }

    fn async_handle(&mut self) -> &mut tokio::runtime::Handle {
        &mut self.systems.async_handle
    }

    fn set_state<T: 'static>(&mut self, value: T) {
        self.state.set::<T>(value);
    }

    fn get_state<T: 'static>(&self) -> Option<&T> {
        self.state.get::<T>()
    }

    fn get_state_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.state.get_mut::<T>()
    }

    fn remove_state<T: 'static>(&mut self) {
        self.state.remove::<T>();
    }

    fn range(&mut self, range: std::ops::Range<i32>) -> i32 {
        self.rng.range(range)
    }
}
impl<'a> EngineContext<'a> {
    pub fn set_fixed_update(&mut self, value: bool) {
        self.is_fixed_update = value;
    }
}

impl<'a> AudioApi for EngineContext<'a> {
    fn play(&mut self, sound: Handler<AudioAsset>, looped: bool) -> Player {
        self.systems
            .audio
            .play_controled(&self.systems.resources, sound, looped)
    }
}
impl<'a> AssetApi for EngineContext<'a> {
    fn load_texture(&mut self, owner: Id, path: &str) -> Handler<ImageAsset> {
        if let Some(id) = self.systems.resources.textures.get_id(path) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file(path);
        self.systems
            .resources
            .textures
            .insert(owner, path, texture_asset)
    }

    fn load_texture_and_resize(
        &mut self,
        owner: Id,
        path: &str,
        width: u32,
        height: u32,
    ) -> Handler<ImageAsset> {
        let path_key = path.to_string() + format!("#{width}x{height}").as_str();
        if let Some(id) = self.systems.resources.textures.get_id(&path_key) {
            return Handler::new(id);
        }

        let texture_asset = ImageAsset::load_from_file_and_resize(path, width, height);
        self.systems
            .resources
            .textures
            .insert(owner, &path_key, texture_asset)
    }

    fn load_audio(&mut self, owner: Id, path: &str) -> Handler<AudioAsset> {
        if let Some(id) = self.systems.resources.sounds.get_id(path) {
            return Handler::new(id);
        }
        let sound_asset = AudioAsset::load_audio(path);
        self.systems
            .resources
            .sounds
            .insert(owner, path, sound_asset)
    }

    fn unload_texture(&mut self, owner: Id, texture: Handler<ImageAsset>) {
        self.systems.resources.textures.remove(owner, texture);
    }

    fn unload_audio(&mut self, owner: Id, audio: Handler<AudioAsset>) {
        self.systems.resources.sounds.remove(owner, audio);
    }

    fn clear_assets(&mut self) {
        let assets = &mut self.systems.resources;
        assets.clear();
    }
}
impl<'a> InputApi for EngineContext<'a> {
    fn is_key_pressed(&self, key: KeyCode) -> bool {
        self.systems.input.is_key_pressed(key)
    }

    fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.systems
            .input
            .is_key_just_pressed(key, self.is_fixed_update)
    }

    fn mouse_position(&self) -> Vector2 {
        self.systems.input.mouse_position()
    }

    fn is_action_pressed(&self, action: &str) -> bool {
        self.systems.input.is_action_pressed(action)
    }

    fn is_action_just_pressed(&self, action: &str) -> bool {
        self.systems
            .input
            .is_action_just_pressed(action, self.is_fixed_update)
    }

    fn get_vector(
        &self,
        action_up: &str,
        action_down: &str,
        action_left: &str,
        action_right: &str,
    ) -> Vector2 {
        self.systems
            .input
            .get_vector(action_up, action_down, action_left, action_right)
    }

    fn get_key_vector(
        &self,
        key_up: KeyCode,
        key_down: KeyCode,
        key_left: KeyCode,
        key_right: KeyCode,
    ) -> Vector2 {
        self.systems
            .input
            .get_key_vector(key_up, key_down, key_left, key_right)
    }

    fn is_mouse_pressed(&self, key: winit::event::MouseButton) -> bool {
        self.systems.input.is_mouse_pressed(key)
    }

    fn is_mouse_just_pressed(&self, key: winit::event::MouseButton) -> bool {
        self.systems
            .input
            .is_mouse_just_pressed(key, self.is_fixed_update)
    }

    fn get_key_axis(&self, negative_key: KeyCode, positive_key: KeyCode) -> f32 {
        self.systems.input.get_key_axis(negative_key, positive_key)
    }

    fn get_axis(&self, negative_action: &str, positive_action: &str) -> f32 {
        self.systems
            .input
            .get_axis(negative_action, positive_action)
    }
}

impl<'a> EventApi for EngineContext<'a> {
    /// Utilizado para enviar uma mensagem endereçada para um GameObject especifico
    /// A mensagem tem que ser do mesmo tipo que o definido em type Message = T;
    fn send<T: Send + 'static>(&mut self, id: Id, message: T) {
        let event = GlobalEvent::Send(id, Box::new(message));
        self.events.insert_global_event(event);
    }
    /// Utilizado para emitir um evento global que sera recebido por todos os GameObjects que definiram um #[game(subscribe(metodo: Tipo))]
    fn emit<T: Send + Sync + 'static>(&mut self, event: T) {
        self.events.insert_broadcast_event(Arc::new(event));
    }
    /// Envia um evento similar a mensagem mas que pode ser de qualquer tipo, geralmente utilizado para comunicação de Componente para GameObject
    fn emit_targeted<T: Send + 'static>(&mut self, id: Id, event: T) {
        let event = GlobalEvent::Targeted(id, Box::new(event));
        self.events.insert_global_event(event);
    }

    fn send_boxed_any(&mut self, id: Id, message: Box<dyn Any + Send + 'static>) {
        let event = GlobalEvent::Send(id, message);
        self.events.insert_global_event(event);
    }

    fn send_service<T: 'static, E: Send + 'static>(&mut self, event: E) {
        if let Some(id) = self.service_id::<T>() {
            self.send(id, event);
        }
    }

    fn take_mailbox(&mut self, id: Id) -> Option<Vec<GlobalEvent>> {
        self.events.take_mailbox(id)
    }

    fn register_subscriptions(&mut self, id: Id, types: &[TypeId]) {
        self.events.register_subscriptions(id, types);
    }

    fn unregister_subscriptions(&mut self, id: Id, types: &[TypeId]) {
        self.events.unregister_subscriptions(id, types);
    }

    fn broadcast_range(&mut self, id: Id, type_id: TypeId) -> (usize, usize) {
        self.events.broadcast_range(id, type_id)
    }

    fn get_broadcast(
        &self,
        type_id: TypeId,
        index: usize,
    ) -> Option<Arc<dyn Any + Send + Sync + 'static>> {
        self.events.get_broadcast(type_id, index)
    }
}
impl<'a> CollisionApi for EngineContext<'a> {
    fn update_collider(&mut self, key: ColliderKey, data: ColliderData) {
        self.systems.collision.update_collider(key, data);
    }

    fn remove_collider(&mut self, key: ColliderKey) {
        self.systems.collision.remove_collider(key);
    }

    fn move_and_slide(
        &mut self,
        my_id: Id,
        position: &mut Vector2,
        velocity: &mut Vector2,
    ) -> CollisionFlag {
        let mut pos = Vector2i::from(*position);
        let mut vel = Vector2i::from(*velocity);

        let flags = self
            .systems
            .collision
            .move_and_slide(my_id, &mut pos, &mut vel);

        *position = pos.into();
        *velocity = vel.into();

        flags
    }

    fn snap_to_floor(&mut self, my_id: Id, snap_length: i32) -> Option<i32> {
        self.systems.collision.snap_to_floor(my_id, snap_length)
    }

    fn translate_my_colliders(&mut self, my_id: Id, offset: Vector2i) {
        self.systems
            .collision
            .translate_my_colliders(my_id, offset.into());
    }

    fn update_collider_geometry(
        &mut self,
        key: ColliderKey,
        layer: Layer,
        mask: Layer,
        is_sensor: bool,
        on_way_collision: bool,
        size: (i32, i32),
        fallback_pos: (i32, i32),
    ) {
        self.systems.collision.update_collider_geometry(
            key,
            layer,
            mask,
            is_sensor,
            on_way_collision,
            size,
            fallback_pos,
        );
    }

    fn check_collisions(&mut self, my_id: Id) -> bool {
        self.systems.collision.check_collisions(my_id)
    }

    fn resolve_axis(
        &mut self,
        my_id: Id,
        position: &mut Vector2i,
        velocity: &mut Vector2i,
        is_x_axis: bool,
    ) -> Vector2i {
        self.systems
            .collision
            .resolve_axis(my_id, position, velocity, is_x_axis)
    }

    fn register_trigger_callbacks(
        &mut self,
        key: ColliderKey,
        on_enter: Option<Box<dyn Fn() -> Box<dyn Any + Send + 'static>>>,
        on_exit: Option<Box<dyn Fn() -> Box<dyn Any + Send + 'static>>>,
    ) {
        self.systems
            .trigger_callbacks
            .insert(key, TriggerCallbacks { on_enter, on_exit });
    }
}

impl<'a> WorldApi for EngineContext<'a> {
    fn spawn<T: GameObject + Send + 'static>(&mut self, obj: T) {
        self.emit(SpawnEvent::new(obj));
    }
    fn register_alive(&mut self, id: Id) {
        self.systems.live_ids.insert(id);
    }
    fn unregister_alive(&mut self, id: Id) {
        self.systems.live_ids.remove(&id);
    }
    fn destroy(&mut self, id: Id) {
        self.systems.live_ids.remove(&id);

        self.systems.resources.textures.remove_game_object(id);
        self.systems.resources.sounds.remove_game_object(id);
    }
}

pub struct AsyncContext {
    sender: Sender<GlobalEvent>,
}

impl AsyncContext {
    pub fn emit<T: Any + Send + Sync + 'static>(&self, event: T) {
        let _ = self.sender.send(GlobalEvent::Broadcast(Arc::new(event)));
    }
    pub fn emit_targeted<T: Any + Send + Sync + 'static>(&self, id: Id, event: T) {
        let _ = self.sender.send(GlobalEvent::Targeted(id, Box::new(event)));
    }
    pub fn send<T: Any + Sync + Send + 'static>(&self, id: Id, message: T) {
        let _ = self.sender.send(GlobalEvent::Send(id, Box::new(message)));
    }
}
impl<'a> SceneApi for EngineContext<'a> {
    fn push_scene<T: crate::prelude::Scene + 'static>(&mut self, scene: T) {
        self.events
            .aplication_commands
            .push_back(AppCommands::PushScene(Box::new(scene)));
    }

    fn change_scene<T: crate::prelude::Scene + 'static>(&mut self, scene: T) {
        self.events
            .aplication_commands
            .push_back(AppCommands::ChangeScene(Box::new(scene)));
    }

    fn pop_scene(&mut self) {
        self.events
            .aplication_commands
            .push_back(AppCommands::PopScene);
    }

    fn clear_scene(&mut self) {
        self.events
            .aplication_commands
            .push_back(AppCommands::ClearScenes);
    }
}
