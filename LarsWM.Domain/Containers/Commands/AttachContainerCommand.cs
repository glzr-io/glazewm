using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public class AttachContainerCommand : Command
  {
    public SplitContainer Parent { get; }
    public Container NewChild { get; }

    public AttachContainerCommand(SplitContainer parent, Container newChild)
    {
      Parent = parent;
      NewChild = newChild;
    }
  }
}
