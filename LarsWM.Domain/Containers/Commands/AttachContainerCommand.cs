using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public enum InsertPosition
  {
    START,
    END,
  }

  public class AttachContainerCommand : Command
  {
    public SplitContainer Parent { get; }
    public Container NewChild { get; }
    public InsertPosition InsertPosition { get; }

    public AttachContainerCommand(SplitContainer parent, Container newChild, InsertPosition insertPosition = InsertPosition.END)
    {
      Parent = parent;
      NewChild = newChild;
      InsertPosition = insertPosition;
    }
  }
}
