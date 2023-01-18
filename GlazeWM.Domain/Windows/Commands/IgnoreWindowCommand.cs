using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class IgnoreWindowCommand : Command
  {
    public Window WindowToIgnore { get; }

    public IgnoreWindowCommand(Window windowToIgnore)
    {
      WindowToIgnore = windowToIgnore;
    }
  }
}
