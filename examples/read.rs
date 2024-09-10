use std::sync::mpsc::Receiver;
#[allow(unused_imports)]
use std::sync::Arc;
#[allow(unused_imports)]
use tun_rs::{AbstractDevice, BoxError};

#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd",))]
use tun_rs::Layer;

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
    use std::net::IpAddr;

    let mut config = tun_rs::Configuration::default();

    #[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd",))]
    config.layer(Layer::L2);

    config
        .address_with_prefix(
            "fd6e:14aa:34e3::2".parse::<IpAddr>().unwrap(),
            "ffff:ffff:ffff:ffff::".parse::<IpAddr>().unwrap(),
        )
        //.address_with_prefix((10,0,0,9), 24u8)
        .destination((10, 0, 0, 1))
        .name("utun9")
        .up();

    let dev = Arc::new(tun_rs::create(&config)?);
    let dev_t = dev.clone();
    let _join = std::thread::spawn(move || {
        let mut buf = [0; 4096];
        loop {
            let amount = dev.recv(&mut buf)?;
            println!("{:?}", &buf[0..amount]);
        }
        #[allow(unreachable_code)]
        Ok::<(), BoxError>(())
    });
    //dev_t.set_network_address((10,0,0,88), (255,255,255,0), Some((10,0,0,1)))?;
    quit.recv().expect("Quit error.");
    println!("recv quit!!!!!");
    println!("{:?}", dev_t.address()?);
    println!("{:?}", dev_t.destination()?);
    println!("{:?}", dev_t.netmask()?);
    dev_t.enabled(false)?;
    Ok(())
}
