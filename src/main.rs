use vulkan_engine::core::GameEngine;

fn main() {
    let mut game_engine = GameEngine::new();

    let scene = game_engine.create_scene();
    game_engine.set_active_scene(scene);

    game_engine.run();
    game_engine.destroy();
}
