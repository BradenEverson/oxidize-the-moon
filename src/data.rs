//! Representations for what data we could send over the websocket, including power consumption, 3D
//! camera data and Lidar

use chrono::NaiveDateTime;
use ndarray::{Array2, IxDyn};
use serde::{Deserialize, Serialize};

pub mod handler;

#[derive(Serialize, Deserialize)]
/// Data paired with a timestamp
pub struct Data {
    /// The data contained
    pub data: HandleableData,
    /// Timestamp of that data
    pub timestamp: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
/// The different representations this handleable data may hold
pub enum HandleableData {
    /// Lidar data
    Lidar(Array2<f64>),
    /// 3D Image Data
    Image3D(ndarray::Array<f64, IxDyn>),
    /// A game command
    GameCommand(isize),
}
