using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
{
  public class SetFocusedDescendantCommand : Command
  {
    public Container FocusedDescendant { get; }

    public SetFocusedDescendantCommand(Container focusedDescendant)
    {
      FocusedDescendant = focusedDescendant;
    }
  }
}
