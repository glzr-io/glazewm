using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ChangeFocusedContainerLayoutCommand : Command
  {
    public Layout NewLayout { get; }

    public ChangeFocusedContainerLayoutCommand(Layout newLayout)
    {
      NewLayout = newLayout;
    }
  }
}
