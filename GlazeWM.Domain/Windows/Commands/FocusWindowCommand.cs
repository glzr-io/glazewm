using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class FocusWindowCommand : Command
  {
    public Window Window { get; set; }

    public FocusWindowCommand(Window window)
    {
      Window = window;
    }
  }
}
