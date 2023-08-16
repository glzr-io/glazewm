using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class ToggleTilingDirectionHandler
    : ICommandHandler<ToggleTilingDirectionCommand>
  {
    private readonly Bus _bus;

    public ToggleTilingDirectionHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(ToggleTilingDirectionCommand command)
    {
      var container = command.Container;

      var currentTilingDirection =
        (container as SplitContainer)?.TilingDirection ??
        (container.Parent as SplitContainer).TilingDirection;

      var newTilingDirection =
        currentTilingDirection == TilingDirection.Horizontal
          ? TilingDirection.Vertical
          : TilingDirection.Horizontal;

      _bus.Invoke(new ChangeTilingDirectionCommand(container, newTilingDirection));

      return CommandResponse.Ok;
    }
  }
}
