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

pub enum TopLevelEvent {
    DialEvent(DialEvent),
    ConnectionEvent(ConnectionEvent)
}

pub enum ThreadSignals {
    SendHaptics([u8; 8]),
    End
}