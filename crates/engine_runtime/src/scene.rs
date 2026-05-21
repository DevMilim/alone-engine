use engine_core::GameObjectDispatch;

pub trait Scene {
    fn get_dispatch(&mut self) -> &mut impl GameObjectDispatch;
}
