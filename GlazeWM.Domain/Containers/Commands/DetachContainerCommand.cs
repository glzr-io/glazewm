using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class DetachContainerCommand : Command
  {
    public Container ChildToRemove { get; }

    public DetachContainerCommand(Container childToRemove)
    {
      ChildToRemove = childToRemove;
    }
  }
}
