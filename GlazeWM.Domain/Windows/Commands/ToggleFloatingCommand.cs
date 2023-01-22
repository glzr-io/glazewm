using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ToggleFloatingCommand : Command
  {
    public Window Window { get; }

    public ToggleFloatingCommand(Window window)
    {
      Window = window;
    }
  }
}
