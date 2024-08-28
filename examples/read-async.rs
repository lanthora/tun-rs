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
#[allow(unused_imports)]
use std::sync::Arc;

use tokio::sync::mpsc::Receiver;
#[allow(unused_imports)]
use tun2::{AbstractDevice, BoxError};

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace")).init();
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(1);

    ctrlc2::set_async_handler(async move {
        tx.send(()).await.expect("Signal error");
    })
    .await;

    main_entry(rx).await?;
    Ok(())
}
#[cfg(any(target_os = "ios", target_os = "android",))]
async fn main_entry(_quit: Receiver<()>) -> Result<(), BoxError> {
    unimplemented!()
}
#[cfg(any(
    target_os = "windows",
    target_os = "linux",
    target_os = "macos",
    target_os = "freebsd",
))]

async fn main_entry(mut quit: Receiver<()>) -> Result<(), BoxError> {
    let mut config = tun2::Configuration::default();

    config
        .address_with_prefix((10, 0, 0, 9), 24)
        .destination((10, 0, 0, 1))
        .mtu(tun2::DEFAULT_MTU)
        .up();

    let dev = Arc::new(tun2::create_as_async(&config)?);
    let size = dev.mtu()? as usize + tun2::PACKET_INFORMATION_LENGTH;
    let mut buf = vec![0; size];
    loop {
        tokio::select! {
            _ = quit.recv() => {
                println!("Quit...");
                break;
            }
            len = dev.recv(&mut buf) => {
                println!("len = {len:?}");
                println!("pkt: {:?}", &buf[..len?]);
            }
        };
    }
    Ok(())
}
