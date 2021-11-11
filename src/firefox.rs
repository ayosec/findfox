//! Wrapper to the Firefox remote interface provided via DBus

use dbus::blocking::LocalConnection;
use std::time::Duration;

/// Prefix for the DBus destinations.
const MOZ_DBUS_PREFIX: &str = "org.mozilla.firefox.";

/// Timeout to invoke DBus methods.
const CALL_TIMEOUT: Duration = Duration::from_secs(2);

/// Main DBus destination.
const DBUS_MAIN: &str = "org.freedesktop.DBus";

pub struct Remote {
    dbus: LocalConnection,
}

pub struct Instance {
    pub id: String,
    pub profile: String,
}

impl Remote {
    pub fn new() -> Result<Self, dbus::Error> {
        let dbus = LocalConnection::new_session()?;
        Ok(Remote { dbus })
    }

    /// Find Firefox instances using their DBus destinations.
    ///
    /// The profile name is decoded from the last segment of the destination. It
    /// is expected to be the profile name encoded as Base64. `_` characters
    /// are replaced with `=`. If the original Base64 string contains any of
    /// `+`, `-`, or `/`, it won't be able to decode it.
    pub fn instances(&self) -> Result<impl Iterator<Item = Instance>, dbus::Error> {
        let (names,): (Vec<String>,) = self
            .dbus
            .with_proxy(DBUS_MAIN, "/", CALL_TIMEOUT)
            .method_call(DBUS_MAIN, "ListNames", ())?;

        let instances = names.into_iter().filter_map(|name| {
            name.strip_prefix(MOZ_DBUS_PREFIX).map(|id| {
                let decoded = base64::decode(id.replace('_', "=")).map(String::from_utf8);
                let profile = match decoded {
                    Ok(Ok(s)) => s,
                    _ => id.to_string(),
                };

                Instance {
                    id: id.to_string(),
                    profile,
                }
            })
        });

        Ok(instances)
    }

    /// Send a list of arguments to a Firefox instance.
    pub fn send(self, id: &str, args: impl Iterator<Item = String>) -> Result<(), dbus::Error> {
        let destination = format!("{}{}", MOZ_DBUS_PREFIX, id);
        let data = encode_args(args);

        let () = self
            .dbus
            .with_proxy(&destination, "/org/mozilla/firefox/Remote", CALL_TIMEOUT)
            .method_call("org.mozilla.firefox", "OpenURL", (data,))?;

        Ok(())
    }
}

/// Encode a list of strings in the binary format expected by Firefox.
///
/// This is an implementation detail in Firefox, so it may change from version
/// to version.
fn encode_args(args: impl Iterator<Item = String>) -> Vec<u8> {
    /// Path to send in remote commands.
    const FIXED_PATH: &[u8] = b"/tmp/\0";

    let args: Vec<_> = args.collect();

    // Remote protocol format:
    //
    // - The first 4 bytes is the number of arguments (little-endian).
    // - Next, for every argument, 4 bytes (LE) with the offset of the argument.
    // - Next, the path of the working directory.
    // - Finally, each argument, null-terminated.

    let mut offset = (args.len() + 1) * 4 + FIXED_PATH.len();

    let mut buffer = Vec::with_capacity(512);
    buffer.extend_from_slice(&u32::to_le_bytes(args.len() as u32));

    for item in &args {
        buffer.extend_from_slice(&u32::to_le_bytes(offset as u32));
        offset += item.len() + 1;
    }

    buffer.extend_from_slice(FIXED_PATH);
    for item in args {
        buffer.extend_from_slice(item.as_bytes());
        buffer.push(0);
    }

    buffer
}
