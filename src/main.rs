use benzene::core::{
    GameEngine,
    app::BenzeneApp,
    assets::MaterialHandle,
    ecs::components::{
        Camera3D, MeshRenderer, Name, PointLight3D, Transform3D,
        directional_light_3d::DirectionalLight3D,
    },
};
use nalgebra::{Vector3, Vector4};

fn main() {
    let _app = BenzeneApp::<()>::new((), Box::new(on_init), Box::new(on_new_frame));
}

pub fn on_init(engine: &mut GameEngine, _state: &mut ()) {
    let mut scene = engine.create_scene();

    let ground_texture = engine.load_texture_from_image("assets/textures/cracked-dirt512x512.jpg");
    let leaves_texture = engine.load_texture_from_image("assets/textures/grass/color.jpg");
    let light_texture = engine.load_texture_from_image("assets/textures/marble/color.jpg");

    let ground_material: MaterialHandle = engine.create_material_from_texture(ground_texture);
    let trunk_material: MaterialHandle = engine.create_material_from_texture(ground_texture);
    let leaves_material: MaterialHandle = engine.create_material_from_texture(leaves_texture);
    let light_material: MaterialHandle = engine.create_material_from_texture(light_texture);

    let ground_mesh = engine.load_mesh_from_obj("assets/models/plane.obj");
    let trunk_mesh = engine.load_mesh_from_obj("assets/models/vase-smooth.obj");
    let canopy_mesh = engine.load_mesh_from_obj("assets/models/torus-smooth.obj");

    let camera = scene.spawn_entity();
    scene.add_name_component(camera, Name::new("Camera"));
    scene.add_transform_3d_component(
        camera,
        Transform3D::new(
            Vector3::new(0.0, 2.8, 6.5),
            Vector3::new((-12.0f32).to_radians(), 0.0, 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_camera_3d_component(camera, Camera3D::new_default());
    scene.set_active_camera(camera);

    let fill_light = scene.spawn_entity();
    scene.add_name_component(fill_light, Name::new("Fill Light"));
    scene.add_transform_3d_component(
        fill_light,
        Transform3D::new(
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new((-40.0f32).to_radians(), (-35.0f32).to_radians(), 0.0),
            Vector3::new(1.0, 1.0, 1.0),
        ),
    );
    scene.add_directional_light_3d_component(
        fill_light,
        DirectionalLight3D::new(Vector4::new(0.95, 0.98, 1.0, 0.18)),
    );

    let ground = scene.spawn_entity();
    scene.add_name_component(ground, Name::new("Ground"));
    scene.add_transform_3d_component(
        ground,
        Transform3D::new(
            Vector3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(4.0, 1.0, 4.0),
        ),
    );
    scene.add_mesh_renderer_component(ground, MeshRenderer::new(ground_mesh, Some(ground_material)));

    let tree_trunk = scene.spawn_entity();
    scene.add_name_component(tree_trunk, Name::new("Tree Trunk"));
    scene.add_transform_3d_component(
        tree_trunk,
        Transform3D::new(
            Vector3::new(0.0, 0.0, -5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.35, 1.25, 0.35),
        ),
    );
    scene.add_mesh_renderer_component(
        tree_trunk,
        MeshRenderer::new(trunk_mesh, Some(trunk_material)),
    );

    let canopy_base = scene.spawn_entity();
    scene.add_name_component(canopy_base, Name::new("Canopy Base"));
    scene.add_transform_3d_component(
        canopy_base,
        Transform3D::new(
            Vector3::new(0.0, 2.05, -5.0),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.95, 0.55, 0.95),
        ),
    );
    scene.add_mesh_renderer_component(
        canopy_base,
        MeshRenderer::new(canopy_mesh, Some(leaves_material)),
    );

    let canopy_mid = scene.spawn_entity();
    scene.add_name_component(canopy_mid, Name::new("Canopy Mid"));
    scene.add_transform_3d_component(
        canopy_mid,
        Transform3D::new(
            Vector3::new(0.0, 2.65, -5.0),
            Vector3::new(0.0, 0.7, 0.0),
            Vector3::new(0.72, 0.42, 0.72),
        ),
    );
    scene.add_mesh_renderer_component(
        canopy_mid,
        MeshRenderer::new(canopy_mesh, Some(leaves_material)),
    );

    let canopy_top = scene.spawn_entity();
    scene.add_name_component(canopy_top, Name::new("Canopy Top"));
    scene.add_transform_3d_component(
        canopy_top,
        Transform3D::new(
            Vector3::new(0.0, 3.15, -5.0),
            Vector3::new(0.0, 1.2, 0.0),
            Vector3::new(0.48, 0.30, 0.48),
        ),
    );
    scene.add_mesh_renderer_component(
        canopy_top,
        MeshRenderer::new(canopy_mesh, Some(leaves_material)),
    );

    let light_source = scene.spawn_entity();
    scene.add_name_component(light_source, Name::new("Lamp"));
    scene.add_transform_3d_component(
        light_source,
        Transform3D::new(
            Vector3::new(2.2, 3.6, -3.2),
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(0.16, 0.08, 0.16),
        ),
    );
    scene.add_point_light_3d_component(
        light_source,
        PointLight3D::new(Vector4::new(1.0, 0.92, 0.75, 20.0)),
    );
    scene.add_mesh_renderer_component(
        light_source,
        MeshRenderer::new(canopy_mesh, Some(light_material)),
    );

    engine.set_active_scene(scene);
}

pub fn on_new_frame(_engine: &mut GameEngine, _state: &mut ()) {}
