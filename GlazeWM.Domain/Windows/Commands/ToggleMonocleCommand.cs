using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class ToggleMonocleCommand : Command
  {
    public Window Window { get; }

    public ToggleMonocleCommand(Window window)
    {
      Window = window;
    }
  }
}
