using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class MoveContainerWithinTreeCommand : Command
  {
    public Container Container { get; }
    public Container TargetParent { get; }
    public int TargetIndex { get; }

    public MoveContainerWithinTreeCommand(Container container, Container targetParent, int targetIndex)
    {
      Container = container;
      TargetParent = targetParent;
      TargetIndex = targetIndex;
    }
  }
}
