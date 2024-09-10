//! Representations for what data we could send over the websocket, including power consumption, 3D
//! camera data and Lidar

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

/// A serialable enum for all types of data that may be sent over
#[derive(Serialize, Deserialize)]
pub enum SendableData {
    /// 3D camera data
    PointCloudImage(Vec<Point3D>),
    /// Power consumption data
    PowerConsumption(f32),
}

/// A 3D point in space for 3D camera data
pub type Point3D = (f32, f32, f32);

impl SendableData {
    /// Creates a 3D point cloud from random data
    pub fn fuzz_3d_img(len: usize) -> Self {
        let mut res = vec![];
        let mut rng = thread_rng();

        for _ in 0..len {
            let point = (
                rng.gen_range(-100f32..100f32),
                rng.gen_range(-100f32..100f32),
                rng.gen_range(-100f32..100f32),
            );
            res.push(point);
        }

        Self::PointCloudImage(res)
    }
}
