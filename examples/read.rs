use std::net::Ipv4Addr;
use std::sync::mpsc::Receiver;
#[allow(unused_imports)]
use std::sync::Arc;

#[allow(unused_imports)]
use tun_rs::BoxError;
use tun_rs::DeviceBuilder;
#[cfg(any(target_os = "windows", target_os = "linux", target_os = "freebsd",))]
#[allow(unused_imports)]
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
    #[allow(unused_imports)]
    use std::net::IpAddr;
    let dev = Arc::new(
        DeviceBuilder::new()
            .name("utun7")
            .ipv4(Ipv4Addr::new(10, 0, 0, 12), 24, None)
            // .ipv4(Ipv4Addr::new(10, 0, 0, 2), Ipv4Addr::new(255, 255, 255, 0), None)
            .ipv6(
                "CDCD:910A:2222:5498:8475:1111:3900:2021".parse().unwrap(),
                64,
            )
            .mtu(1400)
            // .ipv6(
            //     "CDCD:910A:2222:5498:8475:1111:3900:2021".parse().unwrap(),
            //     "FFFF:FFFF:FFFF:FFFF:0000:0000:0000:0000".parse::<Ipv6Addr>().unwrap(),
            // )
            // .ipv6_tuple(vec![( "CDCD:910A:2222:5498:8475:1111:3900:2022".parse().unwrap(),64),
            //                ( "CDCD:910A:2222:5498:8475:1111:3900:2023".parse().unwrap(),64)])
            .build_sync()?,
    );

    println!("if_index = {:?}", dev.if_index());
    println!("mtu = {:?}", dev.mtu());
    #[cfg(windows)]
    println!("mtu ipv6 = {:?}", dev.mtu_v6());
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
    Ok(())
}
