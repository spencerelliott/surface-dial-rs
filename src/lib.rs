extern crate hidapi;
mod events;

use hidapi::HidDevice;

use events::DialEvent;

pub struct SurfaceDial<'a> {
    device: Option<HidDevice>,
    subdivisions: u16,
    is_connected: bool,
    on_connect: &'a dyn FnMut(),
    on_disconnect: &'a dyn FnMut(),
    on_event: &'a dyn FnMut(DialEvent)
}

impl SurfaceDial<'_> {
    pub fn new<'a>(on_connect: &'a dyn FnMut(), on_disconnect: &'a dyn FnMut(), on_event: &'a dyn FnMut(DialEvent)) -> SurfaceDial<'a> {
        SurfaceDial { 
            device: None,
            subdivisions: 0,
            is_connected: false,
            on_connect,
            on_disconnect,
            on_event
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
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
