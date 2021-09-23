using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
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
