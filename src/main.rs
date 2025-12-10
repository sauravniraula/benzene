use benzene::{
    core::{
        GameEngine,
        app::BenzeneApp,
        ecs::{
            components::{
                Camera3D, PointLight3D, Structure3D, Transform3D,
                directional_light_3d::DirectionalLight3D, spot_light_3d::SpotLight3D,
            },
            entities::game_object::GameObject,
            systems::rotate_transform_3d,
        },
    },
    log,
};
use nalgebra::{Vector3, Vector4};

pub struct GameState {
    pub camera: GameObject,
    pub spot_light: GameObject,
    pub plane: GameObject,
    pub sun: GameObject,
}

fn main() {
    let app = BenzeneApp::<Option<GameState>>::new(None, Box::new(on_init), Box::new(on_new_frame));

    // game_engine.destroy();
}

pub fn on_init(engine: &mut GameEngine, state: &mut Option<GameState>) {
    log!("On Init");

    let mut scene = engine.create_scene();

    // Textures
    // let grass_texture = engine.load_texture_from_image("assets/textures/grass/color.jpg");
    // let marble_texture = engine.load_texture_from_image("assets/textures/marble/color.jpg");

    // Camera
    let camera_entity = GameObject::new("Camera");
    scene.add_game_object(camera_entity.clone());
    scene.add_camera_3d_component(&camera_entity, Camera3D::new_default());
    scene.set_active_camera(&camera_entity);

    // Sun
    let sun = GameObject::new("Sun");
    scene.add_game_object(sun.clone());
    scene.add_transform_3d_component(
        &sun,
        Transform3D::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.0, 90f32.to_radians(), 45f32.to_radians()),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_directional_light_3d_component(
        &sun,
        DirectionalLight3D::new(Vector4::new(1.0, 0.95, 0.8, 0.6)),
    );

    // Spot light
    let spot_light_entity = GameObject::new("Spot Light");
    scene.add_game_object(spot_light_entity.clone());
    scene.add_transform_3d_component(
        &spot_light_entity,
        Transform3D::new(
            Vector3::new(5.0, 5.0, -5.0),
            Vector3::new(0.0, 90f32.to_radians(), 45f32.to_radians()),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    // scene.add_spot_light_3d_component(
    //     &spot_light_entity,
    //     SpotLight3D::new(Vector4::new(1.0, 1.0, 1.0, 10.0)),
    // );

    // Red Light
    let red_light_entity = GameObject::new("Red Light");
    scene.add_game_object(red_light_entity.clone());
    scene.add_transform_3d_component(
        &red_light_entity,
        Transform3D::new(
            Vector3::new(2.0, 2.0, -5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    // scene.add_point_light_3d_component(
    //     &red_light_entity,
    //     PointLight3D::new(Vector4::new(1.0, 0.0, 0.0, 1.0)),
    // );

    // Plane
    let plane_entity = GameObject::new("Plane");
    let plane_structure = engine.get_structure_3d_from_obj("assets/models/plane.obj");
    // let plane_material = engine.get_material_3d_from_texture(grass_texture);
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
    // scene.add_material_3d_component(&plane_entity, plane_material);

    // Smooth Vase
    let smooth_vase_entity = GameObject::new("Smooth Vase");
    let smooth_vase: Structure3D =
        engine.get_structure_3d_from_obj("assets/models/vase-smooth.obj");
    // let smooth_vase_material = engine.get_material_3d_from_texture(marble_texture);
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
    // scene.add_material_3d_component(&smooth_vase_entity, smooth_vase_material);

    engine.set_active_scene(scene);

    *state = Some(GameState {
        camera: camera_entity,
        spot_light: spot_light_entity,
        plane: plane_entity,
        sun: sun,
    });
}

pub fn on_new_frame(engine: &mut GameEngine, state: &mut Option<GameState>) {
    // let game_state = state.as_ref().unwrap();

    // let dt = engine.frame_time;

    // let scene = engine.get_active_scene();
    // let sun_transform_3d = scene.get_transform_3d_component(&game_state.sun);
    // rotate_transform_3d(
    //     sun_transform_3d,
    //     Vector3::new(0.0, 0.0, 1.0 * dt.as_secs_f32()),
    // );
    // scene.mark_directional_light_3d_dirty();
}
