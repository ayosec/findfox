//! A CLI program to send commands to Firefox instances using its remote
//! protocol over DBus.

mod firefox;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let remote = firefox::Remote::new()?;

    let mut args = std::env::args().skip(1);
    match (args.next().as_deref(), args.next()) {
        (None | Some("list"), None) => {
            for instance in remote.instances()? {
                println!("{}\t{}", instance.id, instance.profile);
            }
        }

        (Some("send"), Some(id)) => remote.send(&id, args)?,

        _ => {
            eprint!("{}", include_str!("../HELP.txt"));
            std::process::exit(1);
        }
    }

    Ok(())
}
