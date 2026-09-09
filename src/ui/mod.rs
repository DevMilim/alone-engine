use crate::core::UiApi;

pub struct UiSystem {}

pub enum UiEvent {}

pub trait Widget {
    fn layout(&mut self, _ctx: &mut impl UiApi) {}

    fn update(&mut self, _ctx: &mut impl UiApi, _delta: f32) {}

    fn event(&mut self, _ctx: &mut impl UiApi, _event: &UiEvent) {}

    fn draw(&mut self, _ctx: &mut impl UiApi) {}
}
