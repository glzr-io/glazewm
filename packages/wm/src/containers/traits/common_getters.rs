use std::{
  cell::{Ref, RefMut},
  collections::VecDeque,
};

use ambassador::delegatable_trait;
use uuid::Uuid;

use crate::{
  containers::{
    Container, ContainerDto, DirectionContainer, TilingContainer,
    WindowContainer,
  },
  monitors::Monitor,
  workspaces::Workspace,
};

#[delegatable_trait]
pub trait CommonGetters {
  /// A unique identifier for the container.
  fn id(&self) -> Uuid;

  fn as_container(&self) -> Container;

  fn as_tiling_container(&self) -> anyhow::Result<TilingContainer>;

  fn as_window_container(&self) -> anyhow::Result<WindowContainer>;

  fn as_direction_container(&self) -> anyhow::Result<DirectionContainer>;

  fn to_dto(&self) -> anyhow::Result<ContainerDto>;

  fn borrow_parent(&self) -> Ref<'_, Option<Container>>;

  fn borrow_parent_mut(&self) -> RefMut<'_, Option<Container>>;

  fn borrow_children(&self) -> Ref<'_, VecDeque<Container>>;

  fn borrow_children_mut(&self) -> RefMut<'_, VecDeque<Container>>;

  fn borrow_child_focus_order(&self) -> Ref<'_, VecDeque<Uuid>>;

  fn borrow_child_focus_order_mut(&self) -> RefMut<'_, VecDeque<Uuid>>;

  /// Gets the parent container, unless this container is the root.
  fn parent(&self) -> Option<Container> {
    self.borrow_parent().clone()
  }

  /// Direct children of this container.
  fn children(&self) -> VecDeque<Container> {
    self.borrow_children().clone()
  }

  /// Number of children that this container has.
  fn child_count(&self) -> usize {
    self.borrow_children().len()
  }

  /// Whether this container has any direct children.
  fn has_children(&self) -> bool {
    !self.borrow_children().is_empty()
  }

  /// Whether this container has any siblings.
  fn has_siblings(&self) -> bool {
    self.siblings().count() > 0
  }

  /// Whether this container is detached from the tree (i.e. it does not
  /// have a parent).
  fn is_detached(&self) -> bool {
    self.borrow_parent().as_ref().is_none()
  }

  /// Index of this container amongst its siblings.
  ///
  /// Returns 0 if the container has no parent.
  fn index(&self) -> usize {
    self
      .borrow_parent()
      .as_ref()
      .and_then(|parent| {
        parent
          .borrow_children()
          .iter()
          .position(|child| child.id() == self.id())
      })
      .unwrap_or(0)
  }

  /// Gets child container with the given ID.
  fn child_by_id(&self, child_id: &Uuid) -> Option<Container> {
    self
      .borrow_children()
      .iter()
      .find(|child| &child.id() == child_id)
      .cloned()
  }

  fn tiling_children(
    &self,
  ) -> Box<dyn Iterator<Item = TilingContainer> + '_> {
    Box::new(
      self
        .children()
        .into_iter()
        .filter_map(|container| container.try_into().ok()),
    )
  }

  fn descendants(&self) -> Descendants {
    Descendants {
      stack: self.children(),
    }
  }

  fn self_and_descendants(&self) -> Descendants {
    let mut stack = self.children();
    stack.push_front(self.as_container());
    Descendants { stack }
  }

  /// Children in order of last focus.
  fn child_focus_order(&self) -> Box<dyn Iterator<Item = Container> + '_> {
    let child_focus_order = self.borrow_child_focus_order();

    Box::new(std::iter::from_fn(move || {
      for child_id in child_focus_order.iter() {
        if let Some(child) = self.child_by_id(child_id) {
          return Some(child);
        }
      }

      None
    }))
  }

  /// Leaf nodes (i.e. windows and workspaces) in order of last focus.
  fn descendant_focus_order(
    &self,
  ) -> Box<dyn Iterator<Item = Container> + '_> {
    let mut stack = Vec::new();
    stack.push(self.as_container());

    Box::new(std::iter::from_fn(move || {
      while let Some(current) = stack.pop() {
        // Get containers that have no children. Descendant also cannot be
        // the container itself.
        if current.id() != self.id() && !current.has_children() {
          return Some(current);
        }

        // Reverse the child focus order so that the first element is
        // pushed last and popped first.
        for focus_child_id in
          current.borrow_child_focus_order().iter().rev()
        {
          if let Some(focus_child) = current.child_by_id(focus_child_id) {
            stack.push(focus_child);
          }
        }
      }

      None
    }))
  }

  fn siblings(&self) -> Box<dyn Iterator<Item = Container> + '_> {
    Box::new(
      self
        .parent()
        .into_iter()
        .flat_map(|parent| parent.children())
        .filter(move |sibling| sibling.id() != self.id()),
    )
  }

  fn self_and_siblings(&self) -> Box<dyn Iterator<Item = Container> + '_> {
    Box::new(
      self
        .parent()
        .into_iter()
        .flat_map(|parent| parent.children()),
    )
  }

  fn prev_siblings(&self) -> Box<dyn Iterator<Item = Container> + '_> {
    Box::new(
      self
        .self_and_siblings()
        .collect::<Vec<_>>()
        .into_iter()
        .take(self.index())
        .rev(),
    )
  }

  fn next_siblings(&self) -> Box<dyn Iterator<Item = Container> + '_> {
    Box::new(
      self
        .self_and_siblings()
        .collect::<Vec<_>>()
        .into_iter()
        .skip(self.index() + 1),
    )
  }

  fn tiling_siblings(
    &self,
  ) -> Box<dyn Iterator<Item = TilingContainer> + '_> {
    Box::new(
      self
        .siblings()
        .filter_map(|container| container.try_into().ok()),
    )
  }

  fn ancestors(&self) -> Ancestors {
    Ancestors {
      start: self.parent().map(Into::into),
    }
  }

  fn self_and_ancestors(&self) -> Ancestors {
    Ancestors {
      start: Some(self.as_container()),
    }
  }

  /// Workspace that this container belongs to.
  ///
  /// Note that this might return the container itself.
  fn workspace(&self) -> Option<Workspace> {
    self
      .self_and_ancestors()
      .find_map(|container| container.as_workspace().cloned())
  }

  /// Monitor that this container belongs to.
  ///
  /// Note that this might return the container itself.
  fn monitor(&self) -> Option<Monitor> {
    self
      .self_and_ancestors()
      .find_map(|container| container.as_monitor().cloned())
  }

  /// Nearest direction container (i.e. split container or workspace) that
  /// this container belongs to.
  ///
  /// Note that this might return the container itself.
  fn direction_container(&self) -> Option<DirectionContainer> {
    self
      .self_and_ancestors()
      .find_map(|container| container.try_into().ok())
  }

  /// Index of this container in parent's child focus order.
  ///
  /// Returns 0 if the container has no parent.
  fn focus_index(&self) -> usize {
    self
      .parent()
      .and_then(|parent| {
        parent
          .borrow_child_focus_order()
          .iter()
          .position(|id| id == &self.id())
      })
      .unwrap_or(0)
  }

  /// Whether this container or a descendant has focus.
  ///
  /// If `end_ancestor` is provided, then the check for focus will be up to
  /// and including the `end_ancestor`.
  fn has_focus(&self, end_ancestor: Option<Container>) -> bool {
    self
      .self_and_ancestors()
      .take_while(|ancestor| {
        end_ancestor
          .as_ref()
          .map_or(true, |end_ancestor| end_ancestor != ancestor)
      })
      .chain(end_ancestor.clone())
      .all(|ancestor| ancestor.focus_index() == 0)
  }
}

/// An iterator over ancestors of a given container.
pub struct Ancestors {
  start: Option<Container>,
}

impl Iterator for Ancestors {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    self.start.take().inspect(|container| {
      self.start = container.parent().map(Into::into);
    })
  }
}

/// An iterator over descendants of a given container.
pub struct Descendants {
  stack: VecDeque<Container>,
}

impl Iterator for Descendants {
  type Item = Container;

  fn next(&mut self) -> Option<Container> {
    if let Some(container) = self.stack.pop_front() {
      self.stack.extend(container.children());
      return Some(container);
    }
    None
  }
}

/// Implements the `CommonGetters` trait for a given struct.
///
/// Expects that the struct has a wrapping `RefCell` containing a struct
/// with an `id`, `parent`, `children`, and `child_focus_order` field.
#[macro_export]
macro_rules! impl_common_getters {
  ($struct_name:ident) => {
    impl CommonGetters for $struct_name {
      fn id(&self) -> Uuid {
        self.0.borrow().id
      }

      fn as_container(&self) -> Container {
        self.clone().into()
      }

      fn as_tiling_container(&self) -> anyhow::Result<TilingContainer> {
        TryInto::<TilingContainer>::try_into(self.as_container())
          .map_err(anyhow::Error::msg)
      }

      fn as_window_container(&self) -> anyhow::Result<WindowContainer> {
        TryInto::<WindowContainer>::try_into(self.as_container())
          .map_err(anyhow::Error::msg)
      }

      fn as_direction_container(
        &self,
      ) -> anyhow::Result<DirectionContainer> {
        TryInto::<DirectionContainer>::try_into(self.as_container())
          .map_err(anyhow::Error::msg)
      }

      fn to_dto(&self) -> anyhow::Result<ContainerDto> {
        self.to_dto()
      }

      fn borrow_parent(&self) -> Ref<'_, Option<Container>> {
        Ref::map(self.0.borrow(), |inner| &inner.parent)
      }

      fn borrow_parent_mut(&self) -> RefMut<'_, Option<Container>> {
        RefMut::map(self.0.borrow_mut(), |inner| &mut inner.parent)
      }

      fn borrow_children(&self) -> Ref<'_, VecDeque<Container>> {
        Ref::map(self.0.borrow(), |inner| &inner.children)
      }

      fn borrow_children_mut(&self) -> RefMut<'_, VecDeque<Container>> {
        RefMut::map(self.0.borrow_mut(), |inner| &mut inner.children)
      }

      fn borrow_child_focus_order(&self) -> Ref<'_, VecDeque<Uuid>> {
        Ref::map(self.0.borrow(), |inner| &inner.child_focus_order)
      }

      fn borrow_child_focus_order_mut(
        &self,
      ) -> RefMut<'_, VecDeque<Uuid>> {
        RefMut::map(self.0.borrow_mut(), |inner| {
          &mut inner.child_focus_order
        })
      }
    }
  };
}
