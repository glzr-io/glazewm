using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class AttachAndResizeContainerCommand : Command
  {
    public Container ChildToAdd { get; }
    public Container TargetParent { get; }
    public int TargetIndex { get; }

    // Insert child as end element if `targetIndex` is not provided.
    public AttachAndResizeContainerCommand(Container childToAdd, Container targetParent)
    {
      ChildToAdd = childToAdd;
      TargetParent = targetParent;
      TargetIndex = targetParent.Children.Count;
    }

    public AttachAndResizeContainerCommand(Container childToAdd, Container targetParent, int targetIndex)
    {
      ChildToAdd = childToAdd;
      TargetParent = targetParent;
      TargetIndex = targetIndex;
    }
  }
}
