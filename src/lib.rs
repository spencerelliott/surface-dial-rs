extern crate hidapi;
mod events;

use std::thread::{self, JoinHandle};

use flume::{Sender, Receiver};
use hidapi::HidDevice;

use events::{DialEvent, ConnectionEvent, TopLevelEvent, ThreadSignals};

pub struct SurfaceDial<'a> where  {
    subdivisions: u16,
    is_connected: bool,
    on_connection_event: &'a dyn Fn(ConnectionEvent),
    on_event: &'a dyn Fn(DialEvent),
    event_thread: Option<JoinHandle<()>>,
    event_rx: Receiver<TopLevelEvent>,
    signal_tx: Sender<ThreadSignals>,
}

impl<'a> SurfaceDial<'a> {
    /// Creates a new `SurfaceDial` structure and begins the process of searching and
    /// connecting to a dial.
    pub fn new() -> SurfaceDial<'a> {
        // Create the producer/consumer for Dial events
        let (event_tx, event_rx) = flume::unbounded::<TopLevelEvent>();
        let (signal_tx, signal_rx) = flume::unbounded::<ThreadSignals>();

        let handler = thread::spawn(move || {
            let mut still_running = false;
            // Create the context for the HID device

            while still_running {
                // Check for thread messages
                for e in signal_rx.iter() {
                    match e {
                        ThreadSignals::End => still_running = false,
                        ThreadSignals::SendHaptics(h) => {

                        }
                    }
                }

                // Try to connect if not connected

                // Process any messages

                // Loop while getting new events
                
                // d.send_feature_report(&haptic_bytes).expect("Could not set up haptics on Surface Dial");
                
                // Send new events
                event_tx.send(TopLevelEvent::ConnectionEvent(ConnectionEvent::Connect)).expect("Could not send message");
            }
        });

        SurfaceDial { 
            subdivisions: 0,
            is_connected: false,
            on_connection_event: &|_c| {},
            on_event: &|_e| {},
            event_thread: Some(handler),
            event_rx,
            signal_tx,
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
        if self.is_connected {
            let haptic_bytes: [u8; 8] = [0x1, (self.subdivisions & 0xff) as u8, ((self.subdivisions >> 8) & 0xff) as u8, 0x0, 0x3, 0x0, 0x0, 0x0];
            self.signal_tx.send(ThreadSignals::SendHaptics(haptic_bytes)).expect("Could not send haptics message to thread");
        }
    }

    pub fn set_connection_handler(&mut self, connection_handler: &'a dyn Fn(ConnectionEvent)) {
        self.on_connection_event = connection_handler;

        if (self.is_connected) {
            (self.on_connection_event)(ConnectionEvent::Connect);
        }
    }

    pub fn set_event_handler(&mut self, event_handler: &'a dyn Fn(DialEvent)) {
        self.on_event = event_handler;
    }

    /// Processes all of the events currently queued in the buffer. This should be called
    /// fairly often to make sure the buffer does not fill up.
    pub fn process(&self) {
        for e in self.event_rx.iter() {
            match e {
                TopLevelEvent::ConnectionEvent(c) => (self.on_connection_event)(c),
                TopLevelEvent::DialEvent(d) => (self.on_event)(d),
                _ => {}
            }
        }
    }
}

impl Drop for SurfaceDial<'_> {
    fn drop(&mut self) {
        // Notify the processing thread that we're done
        self.signal_tx.send(ThreadSignals::End).expect("Could not send end thread message");

        // Wait for the thread to end before finishing up
        if let Some(handle) = self.event_thread.take() {
            handle.join().expect("Could not join processing thread");
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
