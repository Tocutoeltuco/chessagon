mod controller;
mod directions;
mod game;
mod glue;
mod iface;
mod names;
mod network;
mod piece;
mod utils;

use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use glue::{Event, JsEvent};
use iface::InterfacesManager;
use network::Client;
use wasm_bindgen::prelude::*;

macro_rules! attach {
    ($ctx: expr, $obj: expr) => {{
        let mut handler = $obj;
        $ctx.add_listener(Box::new(move |evt| {
            handler.on_event(evt);
        }));
    };};
}

type Handler = Box<dyn FnMut(&Event)>;

#[wasm_bindgen]
#[derive(Clone)]
pub struct Context {
    handlers: Rc<RefCell<Vec<Handler>>>,
    queue: Rc<RefCell<VecDeque<Event>>>,
}

impl Default for Context {
    fn default() -> Self {
        Context::new()
    }
}

#[wasm_bindgen]
impl Context {
    fn new() -> Self {
        Context {
            handlers: Rc::new(RefCell::new(vec![])),
            queue: Rc::new(RefCell::new(VecDeque::new())),
        }
    }

    #[wasm_bindgen]
    pub fn dispatch_empty(&self, evt: JsEvent) {
        self.dispatch(evt, &[]);
    }

    #[wasm_bindgen]
    pub fn dispatch(&self, evt: JsEvent, data: &[u8]) {
        self.handle(Event::from_js(evt, data));
    }

    fn handle(&self, evt: Event) {
        match self.handlers.try_borrow_mut() {
            Ok(mut h) => {
                // Called outside event handler
                for handler in h.iter_mut() {
                    handler(&evt);
                }

                // Resolve event queue
                let opt = self.queue.borrow_mut().pop_front();
                if let Some(evt) = opt {
                    self.handle(evt);
                }
            }
            Err(_) => {
                // Called inside event handler
                // Queue event
                self.queue.borrow_mut().push_back(evt);
            }
        };
    }

    fn add_listener(&self, handler: Handler) {
        self.handlers.borrow_mut().push(handler);
    }
}

#[wasm_bindgen]
pub fn setup() -> Context {
    let ctx = Context::new();
    attach!(ctx, InterfacesManager::new());
    attach!(ctx, Client::new(&ctx));
    ctx
}
