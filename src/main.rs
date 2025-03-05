use enigo::*;
use hidapi::HidApi;
use pedal::{Pedal, Pedals};
use std::{sync::Arc, time::Duration};
use tokio::sync::Mutex;

mod pedal;
mod utils;

#[tokio::main]
async fn main() {
    let pedal_vendor_id = 0x0FD9;
    let pedal_product_id = 0x0086;
    let pedal_usb = HidApi::new()
        .expect("Failed to create HID API instance")
        .open(pedal_vendor_id, pedal_product_id)
        .expect("Failed to open the pedal HID device");

    let mut pedals = Pedals::new();

    let mut pedal_input_buf = [0u8; 64];
    loop {
        match pedal_usb.read(&mut pedal_input_buf) {
            Ok(_) => {
                handle_pedals_input(&mut pedals, pedal_input_buf).await;
            }
            Err(e) => {
                println!("Error reading from pedal: {}", e);
                break;
            }
        }
    }
}

async fn handle_pedals_input(pedals: &mut Pedals, pedal_input_buf: [u8; 64]) {
    let is_left_pressed = pedal_input_buf[4] == 1;
    let is_central_pressed = pedal_input_buf[5] == 1;
    let is_right_pressed = pedal_input_buf[6] == 1;

    // utils::list_all_keyboard_layouts();

    pedals
        .start_update(is_left_pressed, is_central_pressed, is_right_pressed)
        .await;

    handle_left_pedal(pedals.left.clone()).await;
    handle_central_pedal(pedals.central.clone()).await;
    handle_right_pedal(pedals.right.clone()).await;

    pedals.finish_update().await;
}

// 1. Quick left pedal press opens/focuses LibreWolf app
// 2. Holding left pedal switches to Russian layout. Releasing pedal switches to English layout.
async fn handle_left_pedal(pedal: Arc<Mutex<Pedal>>) {
    let mut pedal_lock = pedal.lock().await;

    if pedal_lock.is_just_pressed() {
        let pedal_clone = pedal.clone();
        pedal_lock.set_future(Some(tokio::spawn(async move {
            println!("Waiting to select Russian keyboard layout...");

            tokio::time::sleep(Duration::from_millis(250)).await;

            // For some reason this keyboard layout appears with different names each time
            utils::select_keyboard_layout(
                "org.sil.ukelele.keyboardlayout.t.keylayout.Russian–IlyaBirmanTypography",
            )
            .unwrap_or_else(|_| {
                utils::select_keyboard_layout(
                    "org.sil.ukelele.keyboardlayout.t.russian–ilyabirmantypography",
                )
                .unwrap()
            });

            println!("Selected Russian keyboard layout");

            pedal_clone.lock().await.set_holding(true);
        })));
    }

    if pedal_lock.is_released() {
        if pedal_lock.is_holding() {
            utils::select_keyboard_layout(
                "org.sil.ukelele.keyboardlayout.t.keylayout.English–IlyaBirmanTypography",
            )
            .unwrap_or_else(|_| {
                utils::select_keyboard_layout(
                    "org.sil.ukelele.keyboardlayout.t.english–ilyabirmantypography",
                )
                .unwrap()
            });
            pedal_lock.set_holding(false);
            println!("Selected English keyboard layout");
        } else {
            utils::open_app("Firefox");
        }
        pedal_lock.set_future(None);
    }
}

// 1. Quick central pedal press opens AI chat window in Raycast
async fn handle_central_pedal(pedal: Arc<Mutex<Pedal>>) {
    let pedals_lock = pedal.lock().await;

    if pedals_lock.is_released() {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        enigo.key(Key::Meta, Direction::Press).unwrap();
        enigo.key(Key::Space, Direction::Click).unwrap();
        enigo.key(Key::Meta, Direction::Release).unwrap();
        std::thread::sleep(Duration::from_millis(25));
        enigo.key(Key::Tab, Direction::Click).unwrap();
    }
}

// 1. Quick right pedal press opens/focuses VS Code app
// 2. Holding right pedal switches to Vim Insert mode. Releasing pedal switches to Vim Normal mode.
async fn handle_right_pedal(pedal: Arc<Mutex<Pedal>>) {
    let mut pedal_lock = pedal.lock().await;

    if pedal_lock.is_just_pressed() {
        let pedal_clone = pedal.clone();
        pedal_lock.set_future(Some(tokio::spawn(async move {
            println!("Waiting to enter Vim Insert mode...");

            tokio::time::sleep(Duration::from_millis(250)).await;

            tokio::task::spawn_blocking(move || {
                let mut enigo = Enigo::new(&Settings::default()).unwrap();
                enigo.key(Key::Escape, Direction::Click).unwrap();
                enigo.key(Key::Unicode('i'), Direction::Click).unwrap();
                println!("Entered Vim Insert mode");
            })
            .await
            .unwrap();

            pedal_clone.lock().await.set_holding(true);
        })));
    } else if pedal_lock.is_released() {
        if pedal_lock.is_holding() {
            let mut enigo = Enigo::new(&Settings::default()).unwrap();
            enigo.key(Key::Escape, Direction::Click).unwrap();
            pedal_lock.set_holding(false);
            println!("Exited Vim Insert mode");
        } else {
            utils::open_app("Visual Studio Code");
        }
        pedal_lock.set_future(None);
    }
}
