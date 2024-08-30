//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.

use std::sync::mpsc::Receiver;
use std::sync::Arc;
use tun2::{AbstractDevice, BoxError};

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd",))]
use tun2::Layer;

fn main() -> Result<(), BoxError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let (tx, rx) = std::sync::mpsc::channel();

    let handle = ctrlc2::set_handler(move || {
        tx.send(()).expect("Signal error.");
        true
    })
    .expect("Error setting Ctrl-C handler");

    main_entry(rx)?;
    handle.join().unwrap();
    Ok(())
}
#[cfg(any(target_os = "ios", target_os = "android",))]
fn main_entry(_quit: Receiver<()>) -> Result<(), BoxError> {
    unimplemented!()
}
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
))]
fn main_entry(quit: Receiver<()>) -> Result<(), BoxError> {
    let mut config = tun2::Configuration::default();

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd",))]
    config.layer(Layer::L2);

    config
        .address_with_prefix((10, 0, 0, 39), 24)
        .destination((10, 0, 0, 1))
        .name("tun39")
        .up();

    let dev = Arc::new(tun2::create(&config)?);
    let dev_t = dev.clone();
    let join = std::thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            let amount = dev.recv(&mut buf)?;
            println!("{:?}", &buf[0..amount]);
        }
        #[allow(unreachable_code)]
        Ok::<(), BoxError>(())
    });
    quit.recv().expect("Quit error.");
    dev_t.enabled(false)?;
    join.join().unwrap().unwrap();
    Ok(())
}
