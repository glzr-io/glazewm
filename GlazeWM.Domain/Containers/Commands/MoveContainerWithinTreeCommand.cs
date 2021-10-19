using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class MoveContainerWithinTreeCommand : Command
  {
    public Container Container { get; }
    public Container TargetParent { get; }
    public int TargetIndex { get; }
    public InsertionPosition InsertionPosition { get; }

    public MoveContainerWithinTreeCommand(Container container, Container targetParent, int targetIndex, InsertionPosition insertionPosition)
    {
      Container = container;
      TargetParent = targetParent;
      TargetIndex = targetIndex;
      InsertionPosition = insertionPosition;
    }
  }
}
