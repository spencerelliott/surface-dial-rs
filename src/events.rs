pub enum DialDirection {
    Clockwise,
    Counterclockwise
}

pub enum DialEvent {
    Rotate { direction: DialDirection, velocity: u32 },
    Button { pressed: bool }
}

pub enum ConnectionEvent {
    Connect,
    Disconnect
}