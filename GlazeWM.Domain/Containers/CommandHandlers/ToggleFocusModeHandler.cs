using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class ToggleFocusModeHandler : ICommandHandler<ToggleFocusModeCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly WorkspaceService _workspaceService;

    public ToggleFocusModeHandler(Bus bus, ContainerService containerService, WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(ToggleFocusModeCommand command)
    {
      var currentFocusMode = _containerService.FocusMode;
      var targetFocusMode = currentFocusMode == FocusMode.TILING
        ? FocusMode.FLOATING : FocusMode.TILING;

      var focusedWorkspace = _workspaceService.GetFocusedWorkspace();

      Window windowToFocus;

      if (targetFocusMode == FocusMode.FLOATING)
        // Get the last focused tiling window within the workspace.
        windowToFocus = focusedWorkspace.LastFocusedDescendantOfType(typeof(FloatingWindow))
          as Window;
      else
        // Get the last focused floating window within the workspace.
        windowToFocus = focusedWorkspace.LastFocusedDescendantOfType(typeof(TilingWindow))
          as Window;

      if (windowToFocus == null)
        return CommandResponse.Ok;

      _bus.Invoke(new FocusWindowCommand(windowToFocus));

      return CommandResponse.Ok;
    }
  }
}
