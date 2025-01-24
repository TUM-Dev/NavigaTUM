use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Direction {
    #[default]
    Depart,
    HardLeft,
    Left,
    SlightlyLeft,
    Continue,
    SlightlyRight,
    Right,
    HardRight,
    CircleClockwise,
    CircleCounterclockwise,
    Stairs,
    Elevator,
    UturnLeft,
    UturnRight,
}
