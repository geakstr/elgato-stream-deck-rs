use std::sync::Arc;
use tokio::{sync::Mutex, task::JoinHandle};

pub struct Pedal {
    pressed: bool,
    prev_pressed: bool,
    holding: bool,
    future_handle: Option<JoinHandle<()>>,
}

impl Pedal {
    pub fn new() -> Self {
        Self {
            pressed: false,
            prev_pressed: false,
            holding: false,
            future_handle: None,
        }
    }

    pub fn start_update(&mut self, pressed: bool) {
        if !pressed && self.future_handle.is_some() {
            self.future_handle.as_ref().unwrap().abort();
        }
        self.pressed = pressed;
    }

    pub fn is_just_pressed(&self) -> bool {
        !self.prev_pressed && self.pressed
    }

    pub fn set_future(&mut self, handle: Option<JoinHandle<()>>) {
        self.future_handle = handle;
    }

    pub fn set_holding(&mut self, val: bool) {
        self.holding = val;
    }

    pub fn is_holding(&self) -> bool {
        self.holding
    }

    pub fn is_released(&self) -> bool {
        self.prev_pressed && !self.pressed
    }

    pub fn finish_update(&mut self) {
        self.prev_pressed = self.pressed;
    }
}

pub struct Pedals {
    pub left: Arc<Mutex<Pedal>>,
    pub central: Arc<Mutex<Pedal>>,
    pub right: Arc<Mutex<Pedal>>,
}

impl Pedals {
    pub fn new() -> Self {
        Self {
            left: Arc::new(Mutex::new(Pedal::new())),
            central: Arc::new(Mutex::new(Pedal::new())),
            right: Arc::new(Mutex::new(Pedal::new())),
        }
    }

    pub async fn start_update(&mut self, left: bool, central: bool, right: bool) {
        self.left.lock().await.start_update(left);
        self.central.lock().await.start_update(central);
        self.right.lock().await.start_update(right);
    }

    pub async fn finish_update(&mut self) {
        self.left.lock().await.finish_update();
        self.central.lock().await.finish_update();
        self.right.lock().await.finish_update();
    }
}
