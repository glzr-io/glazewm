using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class SetTilingCommand : Command
  {
    public Window Window { get; }

    public SetTilingCommand(Window window)
    {
      Window = window;
    }
  }
}
