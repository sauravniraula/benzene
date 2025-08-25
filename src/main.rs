use benzene::core::{
    GameEngine,
    ecs::{
        components::{Camera3D, PointLight3D, Structure3D, Transform3D},
        entities::game_object::GameObject,
    },
    primitives::plane::Plane,
};
use nalgebra::{Vector3, Vector4};

fn main() {
    let mut game_engine = GameEngine::new();

    let mut scene = game_engine.create_scene();

    // Create entities
    let camera_entity = GameObject::new("Camera");
    let light_entity = GameObject::new("Light");
    let plane_entity = GameObject::new("Plane");

    // Attach components
    scene.add_game_object(camera_entity.clone(), Transform3D::new_default());
    scene.add_camera_component(&camera_entity, Camera3D::new_default());
    scene.set_active_camera(&camera_entity);

    scene.add_game_object(
        light_entity.clone(),
        Transform3D::new(
            Vector3::new(5.0, 2.0, 5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_point_light_component(
        &light_entity,
        PointLight3D::new(Vector4::new(1.0, 1.0, 1.0, 1.0)),
    );

    let plane_structure: Structure3D = game_engine.get_structure_from_model_builder::<Plane>();
    scene.add_game_object(
        plane_entity.clone(),
        Transform3D::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_structure_3d_component(&plane_entity, plane_structure);

    game_engine.set_active_scene(scene);

    game_engine.run();
    game_engine.destroy();
}
