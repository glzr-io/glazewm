using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ReplaceContainerCommand : Command
  {
    public Container ParentContainer { get; }
    public int ChildIndex { get; }
    public Container ReplacementContainer { get; }

    public ReplaceContainerCommand(Container parentContainer, int childIndex, Container replacementContainer)
    {
      ParentContainer = parentContainer;
      ChildIndex = childIndex;
      ReplacementContainer = replacementContainer;
    }
  }
}
