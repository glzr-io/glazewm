using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ToggleTilingDirectionCommand : Command
  {
    public Container Container { get; }

    public ToggleTilingDirectionCommand(Container container)
    {
      Container = container;
    }
  }
}
