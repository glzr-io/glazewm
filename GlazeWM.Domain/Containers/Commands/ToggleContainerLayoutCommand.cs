using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ToggleContainerLayoutCommand : Command
  {
    public Container Container { get; }

    public ToggleContainerLayoutCommand(Container container)
    {
      Container = container;
    }
  }
}
