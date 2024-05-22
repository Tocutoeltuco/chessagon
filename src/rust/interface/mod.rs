mod names;

use wasm_bindgen::prelude::*;

use crate::{
    chat::Chat,
    glue::{joinResponse, setPlayerName, setScene, Button, Event},
    utils::{new_rng, Gamemode},
    Context,
};
use names::new_name;

#[derive(Debug, PartialEq, Clone, Copy)]
enum Scene {
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

pub struct InterfacesManager {
    gamemode: Gamemode,
    scene: Scene,
    name: String,
    ctx: Context,
}

impl InterfacesManager {
    pub fn new(ctx: &Context) -> Self {
        InterfacesManager {
            gamemode: Gamemode::Solo,
            scene: Scene::Gamemode,
            name: new_name(&mut new_rng()),
            ctx: ctx.clone(),
        }
    }

    fn set_scene(&mut self, scene: Scene) {
        self.scene = scene;
        setScene(self.scene.into());
    }

    fn menu_hidden(&mut self) {
        self.set_scene(match self.scene {
            Scene::Register => {
                self.ctx.handle(Event::Disconnected);
                Scene::Gamemode
            }
            Scene::Online => Scene::Register,
            Scene::Settings => {
                if self.gamemode == Gamemode::Solo {
                    Scene::Gamemode
                } else {
                    Scene::Online
                }
            }
            _ => panic!("wasn't supposed to close this menu"),
        });
    }

    pub fn on_event(&mut self, evt: &Event) {
        match evt {
            Event::Start => {
                setPlayerName(true, self.name.clone());
                self.set_scene(self.scene);
            }
            Event::SetGamemode(mode) => {
                self.gamemode = (*mode).into();
                if self.gamemode == Gamemode::Solo {
                    self.set_scene(Scene::Settings);
                } else {
                    self.set_scene(Scene::Register);
                }
            }
            Event::MenuHidden(_) => {
                self.menu_hidden();
            }
            Event::Register(name) => {
                self.name = name.to_string();
                setPlayerName(true, name.to_string());
                self.set_scene(Scene::Online);
            }
            Event::SetSettings { .. } => {
                self.set_scene(Scene::Canvas);
            }
            Event::GameButtonClick(btn) => {
                if *btn == Button::PlayAgain {
                    self.set_scene(Scene::Settings);
                }
            }
            Event::JoinedRoom { code, is_host } => {
                Chat::join_room(code);
                if !is_host {
                    joinResponse("success".to_owned());
                    self.set_scene(Scene::Canvas);
                } else {
                    self.set_scene(Scene::Settings);
                }
            }
            Event::NetError(_) => {
                joinResponse("not-found".to_owned());
            }
            _ => {}
        };
    }
}
