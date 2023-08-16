using GlazeWM.Domain.Common.Enums;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.Commands
{
  public class ChangeTilingDirectionCommand : Command
  {
    public Container Container { get; }
    public TilingDirection TilingDirection { get; }

    public ChangeTilingDirectionCommand(
      Container container,
      TilingDirection tilingDirection)
    {
      Container = container;
      TilingDirection = tilingDirection;
    }
  }
}
