extern crate hidapi;
mod events;

use hidapi::HidDevice;

use events::{DialEvent, ConnectionEvent};

pub struct SurfaceDial<'a> where  {
    device: Option<HidDevice>,
    subdivisions: u16,
    is_connected: bool,
    on_connection_event: &'a dyn FnMut(ConnectionEvent),
    on_event: &'a dyn FnMut(DialEvent)
}

impl<'a> SurfaceDial<'a> {
    /// Creates a new `SurfaceDial` structure and begins the process of searching and
    /// connecting to a dial.
    pub fn new() -> SurfaceDial<'a> {
        SurfaceDial { 
            device: None,
            subdivisions: 0,
            is_connected: false,
            on_connection_event: &|c| {},
            on_event: &|e| {}
        }
    }

    /// Sets the subdivisions on any connected Surface Dials. The Surface dial can be split
    /// into a maximum of 3600 subdivisions. A subdivision is denoted by a short haptic
    /// vibration in the physical dial. 
    /// 
    /// When split into subdivisions, the Surface Dial will only report the direction it is 
    /// moving when the threshold of a subdivision is reached. When the subdivisions are set to
    /// 0, the Surface Dial will report the speed and direction at which the dial is moving but
    /// will not offer any sort of haptic vibration.
    /// 
    /// # Examples
    /// 
    /// ```rust
    /// use surface_dial_rs::SurfaceDial;
    /// 
    /// let mut dial = SurfaceDial::new();
    /// dial.set_subdivisions(20);
    /// ```
    pub fn set_subdivisions(&mut self, subdivisions: u16) {
        self.subdivisions = subdivisions;
        self.send_haptics();
    }

    fn send_haptics(&self) {
        match &self.device {
            Some(d) => {
                let haptic_bytes: [u8; 8] = [0x1, (self.subdivisions & 0xff) as u8, ((self.subdivisions >> 8) & 0xff) as u8, 0x0, 0x3, 0x0, 0x0, 0x0];
                d.send_feature_report(&haptic_bytes).expect("Could not set up haptics on Surface Dial");
            },
            None => {

            }
        }
    }

    pub fn set_connection_handler(&mut self, connection_handler: &'a dyn FnMut(ConnectionEvent)) {
        self.on_connection_event = connection_handler;
    }

    pub fn set_event_handler(&mut self, event_handler: &'a dyn FnMut(DialEvent)) {
        self.on_event = event_handler;
    }
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
