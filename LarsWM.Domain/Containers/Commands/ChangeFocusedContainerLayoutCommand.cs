using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
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
