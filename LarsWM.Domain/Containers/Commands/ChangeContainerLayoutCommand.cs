using LarsWM.Domain.Common.Enums;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.Commands
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
