using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class SetFloatingCommand : Command
  {
    public Window Window { get; }

    public SetFloatingCommand(Window window)
    {
      Window = window;
    }
  }
}
