use crate::{GameObject, Id};

pub trait EngineApi: InputApi + AssetApi + EventApi + RenderApi + AudioApi {
    fn quit(&mut self);
}
pub trait InputApi {}

pub trait AssetApi {}
pub trait EventApi {
    fn send<T: 'static>(&mut self, id: Id, message: T);
    fn emit<T: 'static>(&mut self, event: T);
    fn emit_targeted<T: 'static>(&mut self, id: Id, event: T);
    fn spawn<T: GameObject + 'static>(&mut self, obj: T);
}
pub trait RenderApi {}
pub trait AudioApi {}
