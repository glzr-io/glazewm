using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class DetachAndResizeContainerCommand : Command
  {
    public Container ChildToRemove { get; }

    public DetachAndResizeContainerCommand(Container childToRemove)
    {
      ChildToRemove = childToRemove;
    }
  }
}
