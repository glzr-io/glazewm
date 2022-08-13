using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class SetMinimizedCommand : Command
  {
    public Window Window { get; }

    public SetMinimizedCommand(Window window)
    {
      Window = window;
    }
  }
}
