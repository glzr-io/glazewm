using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ChangeContainerLayoutCommand : Command
  {
    public Container Container { get; }
    public Layout NewLayout { get; }

    public ChangeContainerLayoutCommand(Container container, Layout newLayout)
    {
      Container = container;
      NewLayout = newLayout;
    }
  }
}
