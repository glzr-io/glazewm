using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public class AttachContainerCommand : Command
  {
    public SplitContainer Parent { get; }
    public Container NewChild { get; }
    public int InsertPosition { get; }

    // Insert child as end element if `insertPosition` is not provided.
    public AttachContainerCommand(SplitContainer parent, Container newChild)
    {
      Parent = parent;
      NewChild = newChild;
      InsertPosition = parent.Children.Count;
    }

    public AttachContainerCommand(SplitContainer parent, Container newChild, int insertPosition)
    {
      Parent = parent;
      NewChild = newChild;
      InsertPosition = insertPosition;
    }
  }
}
