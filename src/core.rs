use std::sync::LazyLock;

use uuid::Uuid;

use crate::{Base, EngineContext, GameObjectBase, GlobalEvent, Transform2D};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Id {
    id: Uuid,
}

impl Id {
    pub fn new() -> Self {
        Self { id: Uuid::now_v7() }
    }
}

impl Default for Id {
    fn default() -> Self {
        Self::new()
    }
}

pub trait GameObject: GameObjectBase {
    /// Define o tipo de mensagem que o GameObject pode receber em seu on_message
    type Message;

    /// Executado uma vez ao iniciar a cena
    fn start(&mut self, _ctx: &mut EngineContext) {}
    /// Metodo executado a cada frame do loop
    fn update(&mut self, _ctx: &mut EngineContext, _delta: f32) {}
    /// Metodo responsavel por receber mensagens endereçadas para um GameObject especifico utilizando ctx.send(id, mensagem)
    fn on_message(&mut self, _ctx: &mut EngineContext, _msg: &Self::Message) {}
    /// Metodo executado apos todos os updates do GameObject
    fn late_update(&mut self, _ctx: &mut EngineContext, _delta: f32) {}
    /// Metodo com execução fixa a 60 fps
    fn fixed_update(&mut self, _ctx: &mut EngineContext, _delta: f32) {}
    /// Metodo recomendado para utilizar para desenho quando não quiser utilizar componentes de desenho
    fn draw(&mut self, _ctx: &mut EngineContext, _base: &Base) {}
    /// Metodo chamado quando um GameObject executa o metodo self.queue_free() usado para desalocação de recursos ou configuração ao ser removido da cena
    fn destroy(&mut self, _ctx: &mut EngineContext) {}
}

pub trait Component {
    fn start(&mut self, _ctx: &mut EngineContext, _base: &mut Base) {}
    fn update(&mut self, _ctx: &mut EngineContext, _base: &mut Base, _delta: f32) {}
    fn late_update(&mut self, _ctx: &mut EngineContext, _base: &mut Base, _delta: f32) {}
    fn on_event(&mut self, _ctx: &mut EngineContext, _base: &mut Base, _event: &GlobalEvent) {}
    fn fixed_update(&mut self, _ctx: &mut EngineContext, _base: &mut Base, _delta: f32) {}
    fn draw(&mut self, _ctx: &mut EngineContext, _base: &Base) {}
    fn destroy(&mut self, _ctx: &mut EngineContext, _base: &Base) {}
}

pub trait GameObjectDispatch {
    fn is_pending_removal(&self) -> bool {
        false
    }
    fn dispatch_start(&mut self, ctx: &mut EngineContext, base: &Base);
    fn dispatch_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32);
    fn dispatch_late_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32);
    fn dispatch_fixed_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32);
    fn dispatch_event(&mut self, ctx: &mut EngineContext, event: &GlobalEvent);
    fn dispatch_message(&mut self, ctx: &mut EngineContext);
    fn dispatch_draw(&mut self, ctx: &mut EngineContext, base: &Base);
    fn dispatch_destroy(&mut self, ctx: &mut EngineContext);
}

impl<T: GameObjectDispatch + GameObject> GameObjectDispatch for Vec<T> {
    fn dispatch_start(&mut self, ctx: &mut EngineContext, base: &Base) {
        self.retain_mut(|obj| {
            obj.dispatch_start(ctx, base);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32) {
        self.retain_mut(|obj| {
            obj.dispatch_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_late_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32) {
        self.retain_mut(|obj| {
            obj.dispatch_late_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_fixed_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32) {
        self.retain_mut(|obj| {
            obj.dispatch_fixed_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                return false;
            }
            true
        });
    }

    fn dispatch_draw(&mut self, ctx: &mut EngineContext, base: &Base) {
        for obj in self.iter_mut() {
            obj.dispatch_draw(ctx, base);
        }
    }

    fn dispatch_destroy(&mut self, ctx: &mut EngineContext) {
        for obj in self.iter_mut() {
            obj.dispatch_destroy(ctx);
        }
    }

    fn dispatch_event(&mut self, ctx: &mut EngineContext, event: &GlobalEvent) {
        for obj in self.iter_mut() {
            obj.dispatch_event(ctx, event);
        }
    }

    fn dispatch_message(&mut self, ctx: &mut EngineContext) {
        for obj in self.iter_mut() {
            obj.dispatch_message(ctx);
        }
    }
}

impl<T: GameObjectDispatch + GameObject> GameObjectDispatch for Option<T> {
    fn is_pending_removal(&self) -> bool {
        match self {
            Some(obj) => obj.is_pending_removal(),
            None => false,
        }
    }
    fn dispatch_start(&mut self, ctx: &mut EngineContext, base: &Base) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_start(ctx, base);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_late_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_late_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_fixed_update(&mut self, ctx: &mut EngineContext, base: &Base, delta: f32) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_fixed_update(ctx, base, delta);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_draw(&mut self, ctx: &mut EngineContext, base: &Base) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_draw(ctx, base);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_destroy(&mut self, ctx: &mut EngineContext) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_destroy(ctx);
            if obj.is_pending_removal() {
                obj.dispatch_destroy(ctx);
                *self = None;
            }
        }
    }

    fn dispatch_event(&mut self, ctx: &mut EngineContext, event: &GlobalEvent) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_event(ctx, event);
        }
    }

    fn dispatch_message(&mut self, ctx: &mut EngineContext) {
        if let Some(obj) = self.as_mut() {
            obj.dispatch_message(ctx);
        }
    }
}

static EMPTY_BASE: LazyLock<Base> = LazyLock::new(|| Base::new(Transform2D::EMPTY));
impl<T: GameObject> GameObject for Vec<T> {
    type Message = ();
}
impl<T: GameObjectBase> GameObjectBase for Vec<T> {
    fn base(&self) -> &Base {
        &EMPTY_BASE
    }

    fn base_mut(&mut self) -> &mut Base {
        panic!("Tentativa invalida de acessar base_mut em um Vec<GameObject>")
    }
}
impl<T: GameObjectBase> GameObjectBase for Option<T> {
    fn base(&self) -> &Base {
        match self {
            Some(obj) => obj.base(),
            None => &EMPTY_BASE,
        }
    }

    fn base_mut(&mut self) -> &mut Base {
        match self {
            Some(obj) => obj.base_mut(),
            None => panic!("Tentativa de acessar base_mut em um Option vazio."),
        }
    }
}
impl<T: GameObject> GameObject for Option<T> {
    type Message = ();
}
