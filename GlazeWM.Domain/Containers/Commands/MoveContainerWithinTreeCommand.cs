using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class MoveContainerWithinTreeCommand : Command
  {
    public Container ContainerToMove { get; }
    public Container TargetParent { get; }
    public int TargetIndex { get; }
    public bool ShouldAdjustSize { get; }

    /// <summary>
    /// Insert child as end element if `targetIndex` is not provided.
    /// </summary>
    public MoveContainerWithinTreeCommand(Container containerToMove, Container targetParent, bool shouldAdjustSize)
    {
      ContainerToMove = containerToMove;
      TargetParent = targetParent;
      TargetIndex = targetParent.Children.Count;
      ShouldAdjustSize = shouldAdjustSize;
    }

    public MoveContainerWithinTreeCommand(Container containerToMove, Container targetParent, int targetIndex, bool shouldAdjustSize)
    {
      ContainerToMove = containerToMove;
      TargetParent = targetParent;
      TargetIndex = targetIndex;
      ShouldAdjustSize = shouldAdjustSize;
    }
  }
}
