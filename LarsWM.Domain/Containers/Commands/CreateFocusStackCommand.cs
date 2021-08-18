using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public class CreateFocusStackCommand : Command
  {
    public Container FocusedContainer { get; }

    public CreateFocusStackCommand(Container focusedContainer)
    {
      FocusedContainer = focusedContainer;
    }
  }
}
