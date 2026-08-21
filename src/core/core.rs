use bincode::{Decode, Encode};
use uuid::Uuid;

use crate::core::{Base, EngineApi, GameObjectBase, RenderApi};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Encode, Decode)]
pub struct Id {
    id: [u8; 16],
}

impl Id {
    pub fn new() -> Self {
        Self {
            id: *Uuid::now_v7().as_bytes(),
        }
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

pub trait IComponent<T: Component> {
    fn get_self(&self) -> &T;

    fn get_self_mut(&mut self) -> &mut T;
    fn get_self_and_base_mut(&mut self) -> (&mut T, &mut Base);
}
