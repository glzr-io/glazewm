using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class AttachContainerCommand : Command
  {
    public Container ChildToAdd { get; }
    public Container TargetParent { get; }
    public int TargetIndex { get; }

    /// <summary>
    /// Insert child as end element if `targetIndex` is not provided.
    /// </summary>
    public AttachContainerCommand(Container childToAdd, Container targetParent)
    {
      ChildToAdd = childToAdd;
      TargetParent = targetParent;
      TargetIndex = targetParent.Children.Count;
    }

    public AttachContainerCommand(Container childToAdd, Container targetParent, int targetIndex)
    {
      ChildToAdd = childToAdd;
      TargetParent = targetParent;
      TargetIndex = targetIndex;
    }
  }
}
