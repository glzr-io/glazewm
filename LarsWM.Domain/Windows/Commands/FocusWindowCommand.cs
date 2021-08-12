using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Commands
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
