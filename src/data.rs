//! Representations for what data we could send over the websocket, including power consumption, 3D
//! camera data and Lidar

use serde::{Deserialize, Serialize};

/// A serialable enum for all types of data that may be sent over
#[derive(Serialize, Deserialize)]
pub enum SendData {
    /// 3D camera data
    PointCloudImage(Vec<Point>),
    /// Power consumption data
    PowerConsumption(f32),
}

/// A 3D point in space for 3D camera data
#[derive(Serialize, Deserialize)]
pub struct Point(pub f32, pub f32, pub f32);
