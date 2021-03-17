//! Implementation of the Firefox remote protocol, via properties in X11
//! windows.
//!
//! This is an implementation detail in Firefox, so it may changes from version
//! to version.

use crate::x11::Client;
use crate::Error;

use xcb::ffi::xproto::XCB_PROP_MODE_REPLACE;

/// Path to send in remote commands.
const FIXED_PATH: &[u8] = b"/tmp/\0";

/// Parse arguments from the command line to send a remote command.
pub fn send(client: Client, mut args: impl Iterator<Item = String>) -> Result<(), Error> {
    let window = match args.next() {
        Some(a) => match a.strip_prefix("0x") {
            None => a.parse()?,
            Some(a) => u32::from_str_radix(a, 16)?,
        },

        None => return Err("Missing <WINDOW>".into()),
    };

    let items: Vec<_> = args.collect();

    // Remote protocol format:
    //
    // - The first 4 bytes is the number of arguments (little-endian).
    // - Next, for every argument, 4 bytes (LE) with the offset of the argument.
    // - Next, the path of the working directory.
    // - Finally, each argument, null-terminated.

    let mut offset = (items.len() + 1) * 4 + FIXED_PATH.len();

    let mut buffer = Vec::with_capacity(512);
    buffer.extend_from_slice(&u32::to_le_bytes(items.len() as u32));

    for item in &items {
        buffer.extend_from_slice(&u32::to_le_bytes(offset as u32));
        offset += item.len() + 1;
    }

    buffer.extend_from_slice(FIXED_PATH);
    for item in items {
        buffer.extend_from_slice(item.as_bytes());
        buffer.push(0);
    }

    // Write the _MOZILLA_COMMANDLINE property to send the command.
    xcb::xproto::change_property(
        &client.conn,
        XCB_PROP_MODE_REPLACE as u8,
        window,
        client.atom("_MOZILLA_COMMANDLINE")?,
        client.string_type,
        8,
        &buffer,
    )
    .request_check()?;

    Ok(())
}
