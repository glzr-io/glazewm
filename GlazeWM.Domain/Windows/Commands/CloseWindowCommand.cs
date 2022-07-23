using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  public class CloseWindowCommand : Command
  {
    public Window WindowToClose { get; }

    public CloseWindowCommand(Window windowToClose)
    {
      WindowToClose = windowToClose;
    }
  }
}
