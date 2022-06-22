extern crate hidapi;

pub mod events;

use std::{thread::{self, JoinHandle}, borrow::Borrow, time};

use flume::{Sender, Receiver};
use hidapi::{HidDevice, HidApi};

use events::{DialEvent, ConnectionEvent, TopLevelEvent, ThreadSignals};

use crate::events::DialDirection;

const MSFT_VENDOR_ID: u16 = 0x045e;
const SURFACE_DIAL_DEVICE_ID: u16 = 0x091b;

pub struct SurfaceDial<'a> where  {
    subdivisions: u16,
    is_connected: bool,
    on_connection_event: &'a dyn Fn(ConnectionEvent),
    on_event: &'a dyn Fn(DialEvent),
    event_thread: Option<JoinHandle<()>>,
    event_rx: Receiver<TopLevelEvent>,
    signal_tx: Sender<ThreadSignals>,
}

struct DialState {
    button_state: bool
}

impl<'a> SurfaceDial<'a> {
    /// Creates a new `SurfaceDial` structure and begins the process of searching and
    /// connecting to a dial.
    pub fn new() -> SurfaceDial<'a> {
        // Create the producer/consumer for Dial events
        let (event_tx, event_rx) = flume::unbounded::<TopLevelEvent>();
        let (signal_tx, signal_rx) = flume::unbounded::<ThreadSignals>();

        let handler = thread::spawn(move || {
            let mut still_running = true;
            let mut opened_device: Option<HidDevice> = None;

            let mut read_buffer: [u8; 10] = [0; 10];

            let mut dial_state = DialState {
                button_state: false
            };

            let mut current_haptic_bytes: [u8; 8] = [0x1, (0x0 & 0xff) as u8, ((0x0 >> 8) & 0xff) as u8, 0x0, 0x3, 0x0, 0x0, 0x0];

            while still_running {
                // Check for thread messages
                for e in signal_rx.try_iter() {
                    match e {
                        ThreadSignals::End => still_running = false,
                        ThreadSignals::SendHaptics(h) => {
                            current_haptic_bytes = h;
                            
                            if let Some(d) = opened_device.borrow() {
                                // Write haptics
                                d.send_feature_report(&h).expect("Could not set up haptics");
                            }
                        }
                    }
                }

                // Try to connect if not connected
                match opened_device.borrow() {
                    // Create the context for the HID device
                    None => {
                        match HidApi::new() {
                            Ok(api) => {
                                for device in api.device_list() {
                                    if device.vendor_id() == MSFT_VENDOR_ID && device.product_id() == SURFACE_DIAL_DEVICE_ID {
                                        println!("Found Surface Dial!");
                                        let result = device.open_device(&api);

                                        match result {
                                            Ok(d) => {
                                                // Write haptics
                                                d.send_feature_report(&current_haptic_bytes).expect("Could not set up haptics");

                                                opened_device = Some(d);

                                                event_tx.send(TopLevelEvent::ConnectionEvent(ConnectionEvent::Connect)).expect("Could not send message");

                                                break;
                                            },
                                            Err(_e) => {
                                                event_tx.send(TopLevelEvent::ConnectionEvent(ConnectionEvent::Disconnect)).expect("Could not send message");
                                            }
                                        }
                                    }
                                }

                                if let None = opened_device.borrow() {
                                    thread::sleep(time::Duration::from_millis(250));
                                }
                            },
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                        }
                    },
                    // Process any messages
                    Some(d) => {
                        let result = d.read_timeout(&mut read_buffer, 10);

                        match result {
                            Ok(size) => {
                                if size > 0 && read_buffer[0] == 0x1 {
                                    // Check whether the button was pressed (either 0x1 or 0x3 is returned in the button byte)
                                    let button_pressed = if read_buffer[1] == 0x1 || read_buffer[1] == 0x3 {
                                        true
                                    } else {
                                        false
                                    };

                                    // Send a new event depending on the previous state of the button
                                    if button_pressed != dial_state.button_state {
                                        dial_state.button_state = button_pressed;
                                        event_tx.send(TopLevelEvent::DialEvent(DialEvent::Button { pressed: button_pressed })).expect("Could not send message");
                                    }

                                    // Make sure there was enough read from the button and the velocity actually changes
                                    if size > 2 && read_buffer[2] > 0 {
                                        // The velocity goes from -127 to 127 so we have to convert the byte into a signed value
                                        let adjusted_velocity: i16 = if read_buffer[2] > 128 {
                                            read_buffer[2] as i16 - 256i16
                                        } else {
                                            read_buffer[2] as i16
                                        };

                                        // Determine the direction of the dial based on the velocity
                                        let direction: DialDirection = if read_buffer[2] <= 128 {
                                            DialDirection::Clockwise
                                        } else {
                                            DialDirection::Counterclockwise
                                        };

                                        // Send the velocity event
                                        event_tx.send(TopLevelEvent::DialEvent(DialEvent::Rotate { direction: direction, velocity: adjusted_velocity })).expect("Could not send message");
                                    }
                                }
                            },
                            Err(_e) => {
                                opened_device = None;
                                event_tx.send(TopLevelEvent::ConnectionEvent(ConnectionEvent::Disconnect)).expect("Could not send message");
                            }
                        } 
                    }
                }
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
        for e in self.event_rx.try_iter() {
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
