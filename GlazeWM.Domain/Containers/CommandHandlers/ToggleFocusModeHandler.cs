using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class ToggleFocusModeHandler : ICommandHandler<ToggleFocusModeCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private readonly WorkspaceService _workspaceService;

    public ToggleFocusModeHandler(
      Bus bus,
      ContainerService containerService,
      WorkspaceService workspaceService)
    {
      _bus = bus;
      _containerService = containerService;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(ToggleFocusModeCommand command)
    {
      var currentFocusMode = _containerService.FocusMode;
      var targetFocusMode = currentFocusMode == FocusMode.Tiling
        ? FocusMode.Floating
        : FocusMode.Tiling;

      var windowToFocus = GetWindowToFocus(targetFocusMode);

      if (windowToFocus is null)
        return CommandResponse.Ok;

      _bus.Invoke(new SetNativeFocusCommand(windowToFocus));

      return CommandResponse.Ok;
    }

    private Window GetWindowToFocus(FocusMode targetFocusMode)
    {
      var focusedWorkspace = _workspaceService.GetFocusedWorkspace();

      if (targetFocusMode == FocusMode.Floating)
        // Get the last focused tiling window within the workspace.
        return focusedWorkspace.LastFocusedDescendantOfType<FloatingWindow>() as Window;

      // Get the last focused floating window within the workspace.
      return focusedWorkspace.LastFocusedDescendantOfType<TilingWindow>() as Window;
    }
  }
}
