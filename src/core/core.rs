use std::{any::Any, sync::LazyLock};

use uuid::Uuid;

use crate::{Base, EngineApi, GameObjectBase, RenderApi, Transform2D};

pub enum GlobalEvent {
    Broadcast(Box<dyn Any>),
    Targeted(Id, Box<dyn Any>),
}

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
    fn start(&mut self, _ctx: &mut impl EngineApi) {}
    /// Metodo executado a cada frame do loop
    fn update(&mut self, _ctx: &mut impl EngineApi, _delta: f32) {}
    /// Metodo responsavel por receber mensagens endereçadas para um GameObject especifico utilizando ctx.send(id, mensagem)
    fn on_message(&mut self, _ctx: &mut impl EngineApi, _msg: &Self::Message) {}
    /// Metodo executado apos todos os updates do GameObject
    fn late_update(&mut self, _ctx: &mut impl EngineApi, _delta: f32) {}
    /// Metodo com execução fixa a 60 fps
    fn fixed_update(&mut self, _ctx: &mut impl EngineApi, _delta: f32) {}
    /// Metodo recomendado para utilizar para desenho quando não quiser utilizar componentes de desenho
    fn draw(&mut self, _renderer: &mut impl RenderApi, _blending: f32) {}
    /// Metodo chamado quando um GameObject executa o metodo self.queue_free() usado para desalocação de recursos ou configuração ao ser removido da cena
    fn destroy(&mut self, _ctx: &mut impl EngineApi) {}
}

pub trait Component {
    fn start(&mut self, _ctx: &mut impl EngineApi, _base: &mut Base) {}
    fn update(&mut self, _ctx: &mut impl EngineApi, _base: &mut Base, _delta: f32) {}
    fn late_update(&mut self, _ctx: &mut impl EngineApi, _base: &mut Base, _delta: f32) {}
    fn fixed_update(&mut self, _ctx: &mut impl EngineApi, _base: &mut Base, _delta: f32) {}
    fn draw(&mut self, _renderer: &mut impl RenderApi, _base: &Base, _blending: f32) {}
    fn destroy(&mut self, _ctx: &mut impl EngineApi, _base: &Base) {}
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

impl<T: Component> Component for Option<T> {
    fn start(&mut self, ctx: &mut impl EngineApi, base: &mut Base) {
        if let Some(component) = self {
            component.start(ctx, base);
        }
    }
    fn update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if let Some(component) = self {
            component.update(ctx, base, delta);
        }
    }
    fn late_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if let Some(component) = self {
            component.late_update(ctx, base, delta);
        }
    }
    fn fixed_update(&mut self, ctx: &mut impl EngineApi, base: &mut Base, delta: f32) {
        if let Some(component) = self {
            component.fixed_update(ctx, base, delta);
        }
    }
    fn draw(&mut self, renderer: &mut impl RenderApi, base: &Base, blending: f32) {
        if let Some(component) = self {
            component.draw(renderer, base, blending);
        }
    }
    fn destroy(&mut self, ctx: &mut impl EngineApi, base: &Base) {
        if let Some(component) = self {
            component.destroy(ctx, base);
        }
    }
}
