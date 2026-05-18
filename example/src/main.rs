use alone_engine::*;

#[derive(GameObject)]
struct Player {
    #[game(base)]
    base: Base,
    pos: Vector2,
}

impl GameObject for Player {
    type Message = ();
    fn start(&mut self, _ctx: &mut EngineContext) {
        println!("Hello")
    }
}

#[derive(Scene)]
enum GameScenes {
    Main(Player),
}

fn main() {
    let mut engine: Engine<GameScenes> = Engine::new(GameScenes::Main(Player {
        base: Base::new(Transform2D::EMPTY),
        pos: Vector2::ZERO,
    }));
    let mut is_running = true;
    while is_running {
        is_running = engine.step()
    }
}
