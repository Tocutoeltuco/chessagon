use crate::glue::addChatMessage;

pub struct Chat {}

impl Chat {
    pub fn player_message(is_light: bool, name: &str, content: &str) {
        addChatMessage(
            if is_light { 0 } else { 1 },
            vec![name.to_owned(), content.to_owned()],
        );
    }

    pub fn join_room(code: &str) {
        addChatMessage(2, vec![code.to_owned()]);
    }

    pub fn game_start() {
        addChatMessage(3, vec![]);
    }

    pub fn game_end(won_light: bool) {
        addChatMessage(if won_light { 4 } else { 5 }, vec![]);
    }

    pub fn timer_expired(is_light: bool) {
        addChatMessage(if is_light { 6 } else { 7 }, vec![]);
    }

    pub fn resign(is_light: bool) {
        addChatMessage(if is_light { 8 } else { 9 }, vec![]);
    }
}
