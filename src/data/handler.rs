//! A generic Data handling trait for consuming messages coming in from the socket

use super::Data;

/// A generic handler for any data type
pub trait DataHandler {
    /// Handle incoming data
    fn handle(&self, data: Data);
}
