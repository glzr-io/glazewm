using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ReplaceContainerCommand : Command
  {
    public Container ReplacementContainer { get; }
    public Container TargetParent { get; }
    public int TargetIndex { get; }

    public ReplaceContainerCommand(
      Container replacementContainer,
      Container targetParent,
      int targetIndex)
    {
      ReplacementContainer = replacementContainer;
      TargetParent = targetParent;
      TargetIndex = targetIndex;
    }
  }
}
