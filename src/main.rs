use benzene::core::{GameEngine, camera::Camera, primitives::plane::Plane};
use nalgebra::Vector3;

fn main() {
    let mut game_engine = GameEngine::new();

    let mut scene = game_engine.create_scene();
    scene.attach_camera(Camera::new());

    let smooth_vase_model = game_engine.get_game_object_from_obj("assets/models/vase-smooth.obj");
    let mut flat_vase_model = game_engine.get_game_object_from_obj("assets/models/vase-flat.obj");
    let mut flat_torus_model = game_engine.get_game_object_from_obj("assets/models/torus-flat.obj");
    let mut smooth_torus_model =
        game_engine.get_game_object_from_obj("assets/models/torus-smooth.obj");

    let plane_model = game_engine.get_game_object_from_model_builder::<Plane>();

    flat_vase_model.set_position(Vector3::new(-2.0, 0.0, 0.0));
    flat_torus_model.set_position(Vector3::new(4.0, 0.5, 4.0));
    smooth_torus_model.set_position(Vector3::new(4.0, 0.5, 0.0));

    scene.add_game_object(smooth_vase_model);
    scene.add_game_object(plane_model);
    scene.add_game_object(flat_vase_model);
    scene.add_game_object(flat_torus_model);
    scene.add_game_object(smooth_torus_model);

    game_engine.set_active_scene(scene);

    game_engine.run();

    game_engine.destroy();
}
