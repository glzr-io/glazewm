using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public class DetachContainerCommand : Command
  {
    public SplitContainer Parent { get; }
    public Container ChildToRemove { get; }

    public DetachContainerCommand(SplitContainer parent, Container childToRemove)
    {
      Parent = parent;
      ChildToRemove = childToRemove;
    }
  }
}
