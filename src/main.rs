use vulkan_engine::core::GameEngine;

fn main() {
    let mut game_engine = GameEngine::new();
    game_engine.run();

    game_engine.destroy();
}
