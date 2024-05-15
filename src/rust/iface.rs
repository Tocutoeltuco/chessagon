use crate::{
    glue::{joinResponse, setPlayerName, setScene, Event},
    names::new_name,
    utils::{Gamemode, Scene},
};

pub struct InterfacesManager {
    gamemode: Gamemode,
    scene: Scene,
    name: String,
}

impl InterfacesManager {
    pub fn new() -> Self {
        InterfacesManager {
            gamemode: Gamemode::Solo,
            scene: Scene::Gamemode,
            name: new_name(),
        }
    }

    fn set_scene(&mut self, scene: Scene) {
        self.scene = scene;
        setScene(self.scene.into());
    }

    fn menu_hidden(&mut self) {
        self.set_scene(match self.scene {
            Scene::Register => Scene::Gamemode,
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
            Event::JoinedRoom { code, is_host } => {
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
