use std::sync::Arc;

use uuid::Uuid;

use crate::{monitors::Monitor, windows::Window, workspaces::Workspace};

use super::{
  container_type::ContainerType, RootContainer, SplitContainer,
};

#[derive(Debug)]
pub enum Container {
  RootContainer(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  SplitContainer(SplitContainer),
  Window(Window),
}

impl Container {
  pub fn inner(&self) -> &InnerContainer {
    match self {
      Self::RootContainer(c) => &c.inner,
      Self::Monitor(c) => &c.inner,
      Self::Workspace(c) => &c.inner,
      Self::SplitContainer(c) => &c.inner,
      Self::Window(c) => &c.inner,
    }
  }

  pub fn inner_mut(&mut self) -> &mut InnerContainer {
    match self {
      Self::RootContainer(c) => &mut c.inner,
      Self::Monitor(c) => &mut c.inner,
      Self::Workspace(c) => &mut c.inner,
      Self::SplitContainer(c) => &mut c.inner,
      Self::Window(c) => &mut c.inner,
    }
  }

  /// Height of the container. Implementation varies by container type.
  fn height(&self) -> u32 {
    match self {
      Self::RootContainer(c) => c.height(),
      Self::Monitor(c) => c.height(),
      Self::Workspace(c) => c.height(),
      Self::SplitContainer(c) => c.height(),
      Self::Window(c) => c.height(),
    }
  }

  /// Width of the container. Implementation varies by container type.
  fn width(&self) -> u32;

  /// X-coordinate of the container. Implementation varies by container type.
  fn x(&self) -> u32;

  /// Y-coordinate of the container. Implementation varies by container type.
  fn y(&self) -> u32;

  /// Unique identifier for the container.
  fn id(&self) -> Uuid {
    self.inner().id
  }

  pub fn parent(&self) -> Option<Arc<Container>> {
    self.inner().parent.clone()
  }

  pub fn set_parent(&mut self, parent: Option<Arc<Container>>) {
    self.inner_mut().parent = parent;
  }

  pub fn children(&self) -> Vec<Arc<Container>> {
    self.inner().children.clone()
  }

  pub fn set_children(&self, children: Vec<Arc<Container>>) {
    self.inner().children = children;
  }

  /// Order of which child containers last had focus.
  pub fn child_focus_order(&self) -> Vec<Arc<Container>> {
    self.inner().child_focus_order.clone()
  }

  pub fn set_child_focus_order(
    &mut self,
    child_focus_order: Vec<Arc<Container>>,
  ) {
    self.inner().child_focus_order = child_focus_order;
  }

  /// Child container that last had focus.
  pub fn last_focused_child(&self) -> Option<Arc<Container>> {
    self.child_focus_order().get(0).cloned()
  }

  /// Index of this container in parent's child focus order.
  pub fn focus_index(&self) -> u32 {
    match &self.inner().parent {
      None => 0,
      Some(p) => p
        .child_focus_order()
        .iter()
        .position(|child| child.id() == self.id())
        .unwrap()
        .try_into()
        .unwrap(),
    }
  }
}

#[derive(Debug)]
pub struct InnerContainer {
  pub id: Uuid,
  pub parent: Option<Arc<Container>>,
  pub children: Vec<Arc<Container>>,
  pub child_focus_order: Vec<Arc<Container>>,
}

impl InnerContainer {
  pub fn new(
    parent: Option<Arc<Container>>,
    children: Vec<Arc<Container>>,
  ) -> Self {
    Self {
      id: Uuid::new_v4(),
      parent,
      children,
      child_focus_order: children,
    }
  }
}

pub trait ContainerVariant {
  /// Derived container type (eg. `ContainerType::Monitor`).
  fn r#type(&self) -> ContainerType;

  /// Height of the container. Implementation varies by container type.
  fn height(&self) -> u32;

  /// Width of the container. Implementation varies by container type.
  fn width(&self) -> u32;

  /// X-coordinate of the container. Implementation varies by container type.
  fn x(&self) -> u32;

  /// Y-coordinate of the container. Implementation varies by container type.
  fn y(&self) -> u32;

  /// A unique identifier for the container.
  fn inner(&self) -> InnerContainer;

  /// Unique identifier for the container.
  fn id(&self) -> Uuid {
    self.inner().id
  }

  fn parent(&self) -> Option<Arc<Container>> {
    self.inner().parent
  }

  fn set_parent(&mut self, parent: Option<Arc<Container>>) {
    self.inner().parent = parent;
  }

  fn children(&self) -> Vec<Arc<Container>> {
    self.inner().children
  }

  /// Order of which child containers last had focus.
  fn child_focus_order(&self) -> Vec<Arc<Container>> {
    self.inner().child_focus_order
  }

  fn set_child_focus_order(
    &mut self,
    child_focus_order: Vec<Arc<Container>>,
  ) {
    self.inner().child_focus_order = child_focus_order;
  }

  /// Child container that last had focus.
  fn last_focused_child(&self) -> Option<Arc<Container>> {
    self.child_focus_order().get(0).cloned()
  }

  /// Index of this container in parent's child focus order.
  fn focus_index(&self) -> u32 {
    match self.inner().parent {
      None => 0,
      Some(p) => p
        .child_focus_order()
        .iter()
        .position(|child| child.id() == self.id())
        .unwrap()
        .try_into()
        .unwrap(),
    }
  }

  // public List<Container> SelfAndSiblings =>
  //   this is RootContainer ? new List<Container>() { this } : Parent.Children;

  // public IEnumerable<Container> Siblings =>
  //   this is RootContainer
  //     ? Array.Empty<Container>()
  //     : Parent.Children.Where(children => children != this);

  // /// Index of this container amongst its siblings.
  // public int Index => this is RootContainer ? 0 : Parent.Children.IndexOf(this);

  // /// Get the last focused descendant by traversing downwards.
  // public Container LastFocusedDescendant
  // {
  //   get
  //   {
  //     var lastFocusedDescendant = LastFocusedChild;

  //     while (lastFocusedDescendant?.LastFocusedChild != null)
  //       lastFocusedDescendant = lastFocusedDescendant.LastFocusedChild;

  //     return lastFocusedDescendant;
  //   }
  // }

  // /// The sibling at the next index to this container.
  // public Container NextSibling => SelfAndSiblings.ElementAtOrDefault(Index + 1);

  // /// The sibling at the previous index to this container.
  // public Container PreviousSibling => SelfAndSiblings.ElementAtOrDefault(Index - 1);

  // // TODO: Rename to SelfAndDescendants and change to getter.
  // public IEnumerable<Container> Flatten()
  // {
  //   return new[] { this }.Concat(Children.SelectMany(children => children.Flatten()));
  // }

  // public IEnumerable<Container> SelfAndAncestors => new[] { this }.Concat(Ancestors);

  // public IEnumerable<Container> Ancestors
  // {
  //   get
  //   {
  //     var parent = Parent;

  //     while (parent != null)
  //     {
  //       yield return parent;
  //       parent = parent.Parent;
  //     }
  //   }
  // }

  // public IEnumerable<Container> SelfAndDescendants => new[] { this }.Concat(Descendants);

  // /// Breadth-first downward traversal from a single container.
  // public IEnumerable<Container> Descendants
  // {
  //   get
  //   {
  //     var queue = new Queue<Container>();

  //     foreach (var child in Children)
  //       queue.Enqueue(child);

  //     while (queue.Count > 0)
  //     {
  //       var current = queue.Dequeue();
  //       yield return current;
  //       foreach (var child in current.Children)
  //         queue.Enqueue(child);
  //     }
  //   }
  // }

  // /// Leaf nodes (ie. windows and workspaces) in order of last focus.
  // public IEnumerable<Container> DescendantFocusOrder
  // {
  //   get
  //   {
  //     var stack = new Stack<Container>();
  //     stack.Push(this);

  //     // Do a depth-first search using child focus order.
  //     while (stack.Count > 0)
  //     {
  //       var current = stack.Pop();

  //       // Get containers that have no children. Descendant also cannot be the container itself.
  //       if (current != this && !current.HasChildren())
  //         yield return current;

  //       // Reverse the child focus order so that the first element is pushed last/popped first.
  //       foreach (var focusChild in current.ChildFocusOrder.AsEnumerable().Reverse())
  //         stack.Push(focusChild);
  //     }
  //   }
  // }

  // public void InsertChild(int targetIndex, Container child)
  // {
  //   Children.Insert(targetIndex, child);
  //   ChildFocusOrder.Add(child);
  //   child.Parent = this;
  // }

  // public void RemoveChild(Container child)
  // {
  //   child.Parent = null;
  //   Children.Remove(child);
  //   ChildFocusOrder.Remove(child);
  // }

  // public bool IsDetached()
  // {
  //   return Parent is null || Index == -1;
  // }

  // public bool HasChildren()
  // {
  //   return Children.Count > 0;
  // }

  // public bool HasSiblings()
  // {
  //   return Siblings.Any();
  // }

  // public Rect ToRect()
  // {
  //   return new Rect()
  //   {
  //     Left = X,
  //     Right = X + Width,
  //     Top = Y,
  //     Bottom = Y + Height,
  //   };
  // }

  // public IEnumerable<Container> ChildrenOfType<T>()
  // {
  //   return Children.Where(container => typeof(T).IsAssignableFrom(container.GetType()));
  // }

  // public IEnumerable<Container> SiblingsOfType<T>()
  // {
  //   return Siblings.Where(container => typeof(T).IsAssignableFrom(container.GetType()));
  // }

  // public IEnumerable<Container> SelfAndSiblingsOfType<T>()
  // {
  //   return SelfAndSiblings.Where(container => typeof(T).IsAssignableFrom(container.GetType()));
  // }

  // public Container NextSiblingOfType<T>()
  // {
  //   return SelfAndSiblings
  //     .Skip(Index + 1)
  //     .FirstOrDefault(container => typeof(T).IsAssignableFrom(container.GetType()));
  // }

  // public Container PreviousSiblingOfType<T>()
  // {
  //   return SelfAndSiblings
  //     .Take(Index)
  //     .Reverse()
  //     .FirstOrDefault(container => typeof(T).IsAssignableFrom(container.GetType()));
  // }

  // /// Get the last focused child that matches the given type.
  // public Container LastFocusedChildOfType<T>()
  // {
  //   return ChildFocusOrder.Find(
  //     container => typeof(T).IsAssignableFrom(container.GetType())
  //   );
  // }

  // /// Get the last focused descendant that matches the given type.
  // public Container LastFocusedDescendantOfType<T>()
  // {
  //   return DescendantFocusOrder.FirstOrDefault(
  //     container => typeof(T).IsAssignableFrom(container.GetType())
  //   );
  // }
}

/// Aproach 1:
#[derive(Debug)]
struct Container {
  parent: Weak<RefCell<Node>>,
  children: VecDeque<Rc<RefCell<Node>>>,
  value: ContainerValue,
}

impl Container {
  /// Height of the container. Implementation varies by container type.
  pub fn height(&self) -> u32 {
    match self.value {
      Self::Monitor(c) => c.height,
      Self::Workspace(c) => c.height,
      Self::Split(c) => c.height(),
      Self::Window(c) => c.height(),
      _ => 0,
    }
  }

  /// Width of the container. Implementation varies by container type.
  pub fn width(&self) -> u32 {
    match self.value {
      Self::Monitor(c) => c.width,
      Self::Workspace(c) => c.width,
      Self::Split(c) => c.width(),
      Self::Window(c) => c.width(),
      _ => 0,
    }
  }

  /// X-coordinate of the container. Implementation varies by container type.
  pub fn x(&self) -> u32 {
    match self.value {
      Self::Monitor(c) => c.x,
      Self::Workspace(c) => c.x,
      Self::Split(c) => c.x(),
      Self::Window(c) => c.x(),
      _ => 0,
    }
  }

  /// Y-coordinate of the container. Implementation varies by container type.
  pub fn y(&self) -> u32 {
    match self.value {
      Self::Monitor(c) => c.y,
      Self::Workspace(c) => c.y,
      Self::Split(c) => c.y(),
      Self::Window(c) => c.y(),
      _ => 0,
    }
  }

  /// Whether the container can be tiled.
  pub fn can_tile(&self) -> bool {
    match self.value {
      Self::Window(c) => c.state == WindowState::Tiling,
      _ => true,
    }
  }
}

enum ContainerValue {
  Root,
  Monitor(MonitorValue),
  Workspace(WorkspaceValue),
  Split(SplitValue),
  Window(WindowValue),
}

/// Approach 2 ****:
#[derive(Debug)]
pub enum Container {
  RootContainer(RootContainer),
  Monitor(Monitor),
  Workspace(Workspace),
  SplitContainer(SplitContainer),
  Window(Window),
}

pub struct InnerContainer {
  parent: Weak<RefCell<Node>>,
  children: VecDeque<Rc<RefCell<Node>>>,
}

impl Container {
  pub fn inner(&self) -> &InnerContainer {
    match self {
      Self::RootContainer(c) => &c.inner,
      Self::Monitor(c) => &c.inner,
      Self::Workspace(c) => &c.inner,
      Self::SplitContainer(c) => &c.inner,
      Self::Window(c) => &c.inner,
    }
  }
}

#[derive(Debug)]
pub struct RootContainer {
  pub inner: InnerContainer,
  width: u32,
  height: u32,
  x: u32,
  y: u32,
}

impl RootContainer {
  pub fn new() -> Self {
    Self {
      inner: InnerContainer::new(None, vec![]),
      width: 0,
      height: 0,
      x: 0,
      y: 0,
    }
  }
}
