use benzene::core::{
    game_objects::camera::Camera,
    resources::primitives::{cube::Cube, plane::Plane},
    GameEngine,
};

fn main() {
    let mut game_engine = GameEngine::new();

    let mut scene = game_engine.create_scene();
    scene.attach_camera(Camera::new());
    scene.add_model(game_engine.build_model::<Plane>());
    scene.add_model(game_engine.build_model::<Cube>());
    game_engine.set_active_scene(scene);

    game_engine.run();
    game_engine.destroy();
}
