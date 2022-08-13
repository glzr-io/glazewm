using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ToggleMaximizedCommand : Command
  {
    public Window Window { get; }

    public ToggleMaximizedCommand(Window window)
    {
      Window = window;
    }
  }
}
