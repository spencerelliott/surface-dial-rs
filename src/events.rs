#[derive(Debug)]
pub enum DialDirection {
    Clockwise,
    Counterclockwise
}

#[derive(Debug)]
pub enum DialEvent {
    Rotate { direction: DialDirection, velocity: i16 },
    Button { pressed: bool }
}

#[derive(Debug)]
pub enum ConnectionEvent {
    Connect,
    Disconnect
}

#[derive(Debug)]
pub enum TopLevelEvent {
    DialEvent(DialEvent),
    ConnectionEvent(ConnectionEvent)
}

#[derive(Debug)]
pub enum ThreadSignals {
    SendHaptics([u8; 8]),
    End
}