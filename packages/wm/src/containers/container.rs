use std::sync::Arc;

use uuid::Uuid;

use super::container_type::ContainerType;

pub trait Container {
  /// A unique identifier for the container.
  fn id(&self) -> Uuid;

  /// Derived container type (eg. `ContainerType::Monitor`).
  fn r#type(&self) -> ContainerType;

  fn height(&self) -> u32;
  fn width(&self) -> u32;
  fn x(&self) -> u32;
  fn y(&self) -> u32;

  fn parent(&self) -> Option<Arc<dyn Container>>;
  fn set_parent(&mut self, parent: Arc<dyn Container>) -> ();

  fn children(&self) -> Vec<Arc<dyn Container>>;
  fn set_children(&self, children: Vec<Arc<dyn Container>>) -> ();

  /// The order of which child containers last had focus.
  fn child_focus_order(&self) -> Vec<Arc<dyn Container>>;
  fn set_child_focus_order(
    &self,
    child_focus_order: Vec<Arc<dyn Container>>,
  ) -> ();

  /// The child container that last had focus.
  fn last_focused_child(&self) -> Option<Arc<dyn Container>> {
    self.child_focus_order().get(0).cloned()
  }

  // /// Index of this container in parent's child focus order.
  // public int FocusIndex => this is RootContainer ? 0 : Parent.ChildFocusOrder.IndexOf(this);

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
