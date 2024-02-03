using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.Commands
{
  internal class InspectWindowCommand : Command
  {
    public Window WindowToInspect { get; }

    public InspectWindowCommand(Window windowToInspect)
    {
      WindowToInspect = windowToInspect;
    }
  }
}
