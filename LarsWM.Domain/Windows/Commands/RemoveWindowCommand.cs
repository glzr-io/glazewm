using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.Commands
{
  public class RemoveWindowCommand : Command
  {
    public Window Window { get; }

    public RemoveWindowCommand(Window window)
    {
      Window = window;
    }
  }
}
