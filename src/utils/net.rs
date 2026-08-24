use std::net::TcpListener;

pub fn find_available_port(start_port: u16, max_attempts: u16) -> Option<u16> {
    for port in start_port..(start_port + max_attempts) {
        if is_port_available(port) {
            return Some(port);
        }
    }
    None
}

pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}
