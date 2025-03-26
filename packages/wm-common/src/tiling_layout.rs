use ::serde::{Deserialize, Serialize};
use uuid::serde;

use crate::TilingDirection;

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TilingLayout {
  Manual { tiling_direction: TilingDirection },
  MasterStack { master_ratio: f32 },
  Dwindle,
  Grid,
}
