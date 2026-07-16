use std::fs;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TrainerConfig {
    pub pattern_ai_freeze: Vec<i32>,
    pub pattern_network_data: Vec<i32>,
    pub spoofed_division: i32,
    pub spoofed_draft_round: i32,
    pub spoofed_wl_wins: i32,
    pub server_location_id: i32,
}

pub fn load_or_create_config() -> TrainerConfig {
    let config_path = "config.json";
    if let Ok(content) = fs::read_to_string(config_path) {
        if let Ok(config) = serde_json::from_str::<TrainerConfig>(&content) { return config; }
    }
    let new_config = TrainerConfig {
        pattern_ai_freeze: vec![0x48, 0x8B, 0x05, -1, -1, -1, -1, 0x48, 0x8B, 0x88, 0x8B, 0x01],
        pattern_network_data: vec![0x8B, 0x05, -1, -1, -1, -1, 0x89, 0x88, -1, -1, 0x00, 0x00],
        spoofed_division: 1,
        spoofed_draft_round: 3,
        spoofed_wl_wins: 15,
        server_location_id: 14,
    };
    if let Ok(json) = serde_json::to_string_pretty(&new_config) { let _ = fs::write(config_path, json); }
    new_config
}
