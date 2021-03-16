use super::Error;
use std::str;

/// Client for X11.
pub struct Client {
    pub conn: xcb::Connection,
    pub string_type: xcb::Atom,
}

impl Client {
    pub fn new() -> Result<Self, Error> {
        let (conn, _) = xcb::Connection::connect(None)?;

        let string_type = atom(&conn, "STRING")?;

        Ok(Client { conn, string_type })
    }

    pub fn atom(&self, name: &str) -> Result<xcb::Atom, Error> {
        atom(&self.conn, name)
    }

    pub fn get_property(&self, window: xcb::Window, prop: xcb::Atom) -> Option<String> {
        let reply =
            xcb::xproto::get_property(&self.conn, false, window, prop, self.string_type, 0, 1024)
                .get_reply()
                .ok()?;

        if reply.value_len() > 0 {
            str::from_utf8(reply.value()).ok().map(String::from)
        } else {
            None
        }
    }
}

fn atom(conn: &xcb::Connection, name: &str) -> Result<xcb::Atom, Error> {
    Ok(xcb::intern_atom(&conn, false, name).get_reply()?.atom())
}
