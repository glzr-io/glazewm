using System;
using System.Collections.Generic;
using System.Linq;
using GlazeWM.Domain.Common;
using GlazeWM.Infrastructure.WindowsApi;

namespace GlazeWM.Domain.Containers
{
  public abstract class Container
  {
    /// <summary>
    /// A unique identifier for the container.
    /// </summary>
    public Guid Id { get; set; } = Guid.NewGuid();

    /// <summary>
    /// Derived container type (eg. `ContainerType.Monitor`).
    /// </summary>
    public abstract ContainerType Type { get; }

    public virtual int Height { get; set; }
    public virtual int Width { get; set; }
    public virtual int X { get; set; }
    public virtual int Y { get; set; }

    public Container Parent { get; set; }
    public List<Container> Children { get; set; } = new List<Container>();

    /// <summary>
    /// The order of which child containers last had focus.
    /// </summary>
    public List<Container> ChildFocusOrder { get; set; } = new List<Container>();

    /// <summary>
    /// The child container that last had focus.
    /// </summary>
    public Container LastFocusedChild => ChildFocusOrder.ElementAtOrDefault(0);

    /// <summary>
    /// Index of this container in parent's child focus order.
    /// </summary>
    public int FocusIndex => this is RootContainer ? 0 : Parent.ChildFocusOrder.IndexOf(this);

    public List<Container> SelfAndSiblings => Parent.Children;

    public IEnumerable<Container> Siblings => Parent.Children.Where(children => children != this);

    /// <summary>
    /// Index of this container amongst its siblings.
    /// </summary>
    public int Index => Parent.Children.IndexOf(this);

    /// <summary>
    /// Get the last focused descendant by traversing downwards.
    /// </summary>
    public Container LastFocusedDescendant
    {
      get
      {
        var lastFocusedDescendant = LastFocusedChild;

        while (lastFocusedDescendant?.LastFocusedChild != null)
          lastFocusedDescendant = lastFocusedDescendant.LastFocusedChild;

        return lastFocusedDescendant;
      }
    }

    /// <summary>
    /// The sibling at the next index to this container.
    /// </summary>
    public Container NextSibling => SelfAndSiblings.ElementAtOrDefault(Index + 1);

    /// <summary>
    /// The sibling at the previous index to this container.
    /// </summary>
    public Container PreviousSibling => SelfAndSiblings.ElementAtOrDefault(Index - 1);

    // TODO: Rename to SelfAndDescendants and change to getter.
    public IEnumerable<Container> Flatten()
    {
      return new[] { this }.Concat(Children.SelectMany(children => children.Flatten()));
    }

    public IEnumerable<Container> SelfAndAncestors => new[] { this }.Concat(Ancestors);

    public IEnumerable<Container> Ancestors
    {
      get
      {
        var parent = Parent;

        while (parent != null)
        {
          yield return parent;
          parent = parent.Parent;
        }
      }
    }

    public IEnumerable<Container> SelfAndDescendants => new[] { this }.Concat(Descendants);

    /// <summary>
    /// Breadth-first downward traversal from a single container.
    /// </summary>
    public IEnumerable<Container> Descendants
    {
      get
      {
        var queue = new Queue<Container>();

        foreach (var child in Children)
          queue.Enqueue(child);

        while (queue.Count > 0)
        {
          var current = queue.Dequeue();
          yield return current;
          foreach (var child in current.Children)
            queue.Enqueue(child);
        }
      }
    }

    /// <summary>
    /// Leaf nodes (ie. windows and workspaces) in order of last focus.
    /// </summary>
    public IEnumerable<Container> DescendantFocusOrder
    {
      get
      {
        var stack = new Stack<Container>();
        stack.Push(this);

        // Do a depth-first search using child focus order.
        while (stack.Count > 0)
        {
          var current = stack.Pop();

          // Get containers that have no children. Descendant also cannot be the container itself.
          if (current != this && !current.HasChildren())
            yield return current;

          // Reverse the child focus order so that the first element is pushed last/popped first.
          foreach (var focusChild in current.ChildFocusOrder.AsEnumerable().Reverse())
            stack.Push(focusChild);
        }
      }
    }

    public void InsertChild(int targetIndex, Container child)
    {
      Children.Insert(targetIndex, child);
      ChildFocusOrder.Add(child);
      child.Parent = this;
    }

    public void RemoveChild(Container child)
    {
      child.Parent = null;
      Children.Remove(child);
      ChildFocusOrder.Remove(child);
    }

    public bool IsDetached()
    {
      return Parent is null || Index == -1;
    }

    public bool HasChildren()
    {
      return Children.Count > 0;
    }

    public bool HasSiblings()
    {
      return Siblings.Any();
    }

    public Rect ToRect()
    {
      return new Rect()
      {
        Left = X,
        Right = X + Width,
        Top = Y,
        Bottom = Y + Height,
      };
    }

    public IEnumerable<Container> ChildrenOfType<T>()
    {
      return Children.Where(container => typeof(T).IsAssignableFrom(container.GetType()));
    }

    public IEnumerable<Container> SiblingsOfType<T>()
    {
      return Siblings.Where(container => typeof(T).IsAssignableFrom(container.GetType()));
    }

    public IEnumerable<Container> SelfAndSiblingsOfType<T>()
    {
      return SelfAndSiblings.Where(container => typeof(T).IsAssignableFrom(container.GetType()));
    }

    public Container NextSiblingOfType<T>()
    {
      return SelfAndSiblings
        .Skip(Index + 1)
        .FirstOrDefault(container => typeof(T).IsAssignableFrom(container.GetType()));
    }

    public Container PreviousSiblingOfType<T>()
    {
      return SelfAndSiblings
        .Take(Index)
        .Reverse()
        .FirstOrDefault(container => typeof(T).IsAssignableFrom(container.GetType()));
    }

    /// <summary>
    /// Get the last focused child that matches the given type.
    /// </summary>
    public Container LastFocusedChildOfType<T>()
    {
      return ChildFocusOrder.Find(
        container => typeof(T).IsAssignableFrom(container.GetType())
      );
    }

    /// <summary>
    /// Get the last focused descendant that matches the given type.
    /// </summary>
    public Container LastFocusedDescendantOfType<T>()
    {
      return DescendantFocusOrder.FirstOrDefault(
        container => typeof(T).IsAssignableFrom(container.GetType())
      );
    }
  }
}
