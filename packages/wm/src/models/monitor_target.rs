use wm_common::Direction;

use super::Monitor;

pub enum MonitorTarget {
  Index(usize),
  Monitor(Monitor),
  Direction(Direction),
}
