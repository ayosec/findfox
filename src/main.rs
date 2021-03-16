//! List Firefox instances to send remote commands.

/// Expected value for _MOZILLA_VERSION.
const MOZ_VERSION: &str = "5.1";

/// Expected value for _MOZILLA_VERSION.
const MOZ_PROGRAM: &str = "firefox";

/// Generic error type.
type Error = Box<dyn std::error::Error>;

mod docker;
mod x11;

fn main() -> Result<(), Error> {
    let client = x11::Client::new()?;

    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("list") => list_hostnames(client),
        _ => {
            eprint!("{}", include_str!("../HELP.txt"));
            std::process::exit(1);
        }
    }
}

fn list_hostnames(client: x11::Client) -> Result<(), Error> {
    let moz_version = client.atom("_MOZILLA_VERSION")?;
    let moz_program = client.atom("_MOZILLA_PROGRAM")?;
    let hostname = client.atom("WM_CLIENT_MACHINE")?;

    for roots in client.conn.get_setup().roots() {
        let reply = xcb::xproto::query_tree(&client.conn, roots.root()).get_reply()?;
        for &window in reply.children() {
            if client.get_property(window, moz_version).as_deref() == Some(MOZ_VERSION)
                && client.get_property(window, moz_program).as_deref() == Some(MOZ_PROGRAM)
            {
                if let Some(hostname) = client.get_property(window, hostname) {
                    println!("{} {}", docker::get_container_name(&hostname)?, window);
                }
            }
        }
    }

    Ok(())
}
