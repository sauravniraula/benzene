use benzene::core::{
    GameEngine,
    game_objects::camera::Camera,
    resources::primitives::{cube::Cube, plane::Plane},
};

fn main() {
    let mut game_engine = GameEngine::new();

    let image_texture = game_engine.get_image("assets/textures/cracked-dirt512x512.jpg");

    let mut scene = game_engine.create_scene();
    scene.attach_camera(Camera::new());
    scene.add_model(game_engine.build_model::<Plane>());
    scene.add_model(game_engine.build_model::<Cube>());
    scene.add_image(image_texture);

    game_engine.set_active_scene(scene);

    game_engine.run();
    game_engine.destroy();
}
