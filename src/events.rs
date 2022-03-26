pub struct DialEvent {
    pub event_type: DialEventType,
    pub value: i32
}

pub enum DialEventType {
    Rotate,
    Press
}