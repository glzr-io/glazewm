using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public class ReplaceContainerCommand : Command
  {
    public Container ContainerToReplace { get; }
    public Container ReplacementContainer { get; }

    public ReplaceContainerCommand(Container containerToReplace, Container replacementContainer)
    {
      ContainerToReplace = containerToReplace;
      ReplacementContainer = replacementContainer;
    }
  }
}
