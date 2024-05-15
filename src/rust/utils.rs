#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Gamemode {
    Solo,
    Online,
    Bot,
}

impl From<u8> for Gamemode {
    fn from(value: u8) -> Self {
        match value {
            0 => Gamemode::Solo,
            1 => Gamemode::Online,
            2 => Gamemode::Bot,
            _ => panic!("invalid gamemode"),
        }
    }
}

impl From<Gamemode> for u8 {
    fn from(value: Gamemode) -> u8 {
        match value {
            Gamemode::Solo => 0,
            Gamemode::Online => 1,
            Gamemode::Bot => 2,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Scene {
    Loading,
    Canvas,
    Gamemode,
    Register,
    Online,
    Settings,
}

impl From<i8> for Scene {
    fn from(value: i8) -> Self {
        match value {
            -2 => Scene::Loading,
            -1 => Scene::Canvas,
            0 => Scene::Gamemode,
            1 => Scene::Register,
            2 => Scene::Online,
            3 => Scene::Settings,
            _ => panic!("invalid scene"),
        }
    }
}

impl From<Scene> for i8 {
    fn from(value: Scene) -> i8 {
        match value {
            Scene::Loading => -2,
            Scene::Canvas => -1,
            Scene::Gamemode => 0,
            Scene::Register => 1,
            Scene::Online => 2,
            Scene::Settings => 3,
        }
    }
}
