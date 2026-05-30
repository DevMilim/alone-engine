use alone_engine::{Base, GameObject, GameObjectBase, Vector2};

#[derive(GameObject)]
pub struct Player {
    #[game(base)]
    base: Base,
}

impl Player {
    pub fn new(position: Vector2) -> Self {
        Self {
            base: Base::new(position),
        }
    }
}

impl GameObject for Player {
    type Message = ();
}
