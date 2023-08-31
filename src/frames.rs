pub enum InertialFrames {
    Local,
    Global,
}

pub enum TemporalFrames {
    Boot,
    Epoch,
}

pub const ALL_INERTIAL_FRAMES: [InertialFrames; 2] =
    [InertialFrames::Local, InertialFrames::Global];

pub const ALL_TEMPORAL_FRAMES: [TemporalFrames; 2] = [TemporalFrames::Boot, TemporalFrames::Epoch];

impl std::convert::From<i32> for InertialFrames {
    fn from(value: i32) -> Self {
        match value {
            0 => InertialFrames::Local,
            1 => InertialFrames::Global,
            _ => panic!("Unknown inertial frame {}", value),
        }
    }
}

impl std::convert::Into<i32> for InertialFrames {
    fn into(self) -> i32 {
        match self {
            InertialFrames::Local => 0,
            InertialFrames::Global => 1,
        }
    }
}

impl std::convert::From<i32> for TemporalFrames {
    fn from(value: i32) -> Self {
        match value {
            0 => TemporalFrames::Boot,
            1 => TemporalFrames::Epoch,
            _ => panic!("Unknown temporal frame {}", value),
        }
    }
}

impl std::convert::Into<i32> for TemporalFrames {
    fn into(self) -> i32 {
        match self {
            TemporalFrames::Boot => 0,
            TemporalFrames::Epoch => 1,
        }
    }
}
