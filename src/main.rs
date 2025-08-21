use benzene::core::{GameEngine, camera::Camera, primitives::plane::Plane};

fn main() {
    let mut game_engine = GameEngine::new();

    let mut scene = game_engine.create_scene();
    scene.attach_camera(Camera::new());

    let torus_model = game_engine.get_model_from_obj("assets/models/vase-smooth.obj");
    let plane_model = game_engine.build_model::<Plane>();

    scene.add_model(torus_model);
    scene.add_model(plane_model);

    game_engine.set_active_scene(scene);

    game_engine.run();

    game_engine.destroy();
}
