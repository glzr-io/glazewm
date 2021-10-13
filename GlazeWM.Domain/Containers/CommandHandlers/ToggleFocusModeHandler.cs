using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  class ToggleFocusModeHandler : ICommandHandler<ToggleFocusModeCommand>
  {
    private ContainerService _containerService;
    private WorkspaceService _workspaceService;

    public ToggleFocusModeHandler(ContainerService containerService, WorkspaceService workspaceService)
    {
      _containerService = containerService;
      _workspaceService = workspaceService;
    }
    public CommandResponse Handle(ToggleFocusModeCommand command)
    {
      var currentFocusMode = _containerService.FocusMode;
      var targetFocusMode = currentFocusMode == FocusMode.TILING
        ? FocusMode.FLOATING : FocusMode.TILING;

      if (targetFocusMode == FocusMode.TILING)
      {
        // TODO: Get last focused descendant of `TilingWindow` type.
      }

      if (targetFocusMode == FocusMode.FLOATING)
      {
        var focusedWorkspace = _workspaceService.GetFocusedWorkspace();
        // TODO: Get last focused descendant of `FloatingWindow` type.
      }

      return CommandResponse.Ok;
    }
  }
}
