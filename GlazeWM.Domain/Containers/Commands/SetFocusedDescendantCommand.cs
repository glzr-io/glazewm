using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class SetFocusedDescendantCommand : Command
  {
    public Container FocusedDescendant { get; }
    public Container EndAncestor { get; }

    public SetFocusedDescendantCommand(Container focusedDescendant, Container endAncestor = null)
    {
      FocusedDescendant = focusedDescendant;
      EndAncestor = endAncestor;
    }
  }
}
