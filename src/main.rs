use benzene::core::{
    GameEngine,
    ecs::{
        components::{Camera3D, PointLight3D, Structure3D, Transform3D},
        entities::game_object::GameObject,
    },
};
use nalgebra::{Vector3, Vector4};

fn main() {
    let mut game_engine = GameEngine::new();

    let mut scene = game_engine.create_scene();

    // Create entities
    // Camera
    let camera_entity = GameObject::new("Camera");

    // Lights
    let sun = GameObject::new("Sun");
    let red_light_entity = GameObject::new("Red Light");

    // Models
    let plane_entity = GameObject::new("Plane");
    let smooth_vase_entity = GameObject::new("Smooth Vase");

    // Camera (focus plane by default)
    scene.add_game_object(camera_entity.clone());
    scene.add_camera_3d_component(&camera_entity, Camera3D::new_default());
    scene.set_active_camera(&camera_entity);

    // Point Light
    scene.add_game_object(sun.clone());
    scene.add_transform_3d_component(
        &sun,
        Transform3D::new(
            Vector3::new(100.0, 100.0, 100.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_point_light_3d_component(
        &sun,
        PointLight3D::new(Vector4::new(1.0, 0.95, 0.8, 5000.0)),
    );

    scene.add_game_object(red_light_entity.clone());
    scene.add_transform_3d_component(
        &red_light_entity,
        Transform3D::new(
            Vector3::new(2.0, 2.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_point_light_3d_component(
        &red_light_entity,
        PointLight3D::new(Vector4::new(1.0, 0.0, 0.0, 1.0)),
    );

    // Plane
    let plane_structure = game_engine.get_structure_3d_from_obj("assets/models/plane.obj");
    let dirt_texture_id =
        game_engine.load_texture_from_image("assets/textures/cracked-dirt512x512.jpg");
    let plane_material = game_engine.get_material_3d_from_texture_id(dirt_texture_id);
    scene.add_game_object(plane_entity.clone());
    scene.add_transform_3d_component(
        &plane_entity,
        Transform3D::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(2.0, 1.0, 2.0),
        ),
    );
    scene.add_structure_3d_component(&plane_entity, plane_structure);
    scene.add_material_3d_component(&plane_entity, plane_material);

    // Smooth Vase
    let smooth_vase: Structure3D =
        game_engine.get_structure_3d_from_obj("assets/models/vase-smooth.obj");
    scene.add_game_object(smooth_vase_entity.clone());
    scene.add_transform_3d_component(
        &smooth_vase_entity,
        Transform3D::new(
            Vector3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_structure_3d_component(&smooth_vase_entity, smooth_vase);

    game_engine.set_active_scene(scene);

    game_engine.run();
    game_engine.destroy();
}
