use uuid::Uuid;

use crate::containers::ContainerType;

pub trait CommonContainer {
  fn id(&self) -> Uuid;

  fn r#type(&self) -> ContainerType;
}
