use super::Error;
use std::io::{Read, Write};
use unix_socket::UnixStream;

/// Path for Docker socket.
const DOCKER_SOCK: &str = "/var/run/docker.sock";

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Container {
    name: String,
}

/// Get container name from hostname.
pub fn get_container_name(hostname: &str) -> Result<String, Error> {
    let mut stream = UnixStream::connect(DOCKER_SOCK)?;
    write!(
        &mut stream,
        "GET /containers/{}/json HTTP/1.0\r\n\r\n",
        hostname
    )?;

    let mut buffer = Vec::with_capacity(4 * 1024);
    stream.read_to_end(&mut buffer)?;
    let response = std::str::from_utf8(&buffer)?;

    match response.split_ascii_whitespace().nth(1) {
        Some("200") => (),
        Some(code) => return Err(format!("Invalid response status: {}", code).into()),
        None => return Err("Missing status from Docker response.".into()),
    }

    let body_start = match response.find("\r\n\r\n") {
        Some(pos) => pos,
        None => return Err("Missing response body.".into()),
    };

    let c: Container = serde_json::from_str(&response[body_start..])?;
    Ok(c.name.trim_start_matches('/').into())
}
