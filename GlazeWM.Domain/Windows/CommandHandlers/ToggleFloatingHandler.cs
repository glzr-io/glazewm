using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class ToggleFloatingHandler : ICommandHandler<ToggleFloatingCommand>
  {
    private Bus _bus;
    private WorkspaceService _workspaceService;

    public ToggleFloatingHandler(Bus bus, WorkspaceService workspaceService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
    }

    public CommandResponse Handle(ToggleFloatingCommand command)
    {
      var window = command.Window;

      if (!(window is TilingWindow))
        return CommandResponse.Ok;

      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);

      _bus.Invoke(new DetachContainerCommand(window));

      // Create a floating window and place it in the center of the workspace.
      var floatingWindow = new FloatingWindow(window.Hwnd)
      {
        Width = window.OriginalWidth,
        Height = window.OriginalHeight,
        X = workspace.X + (workspace.Width / 2) - (window.OriginalWidth / 2),
        Y = workspace.Y + (workspace.Height / 2) - (window.OriginalHeight / 2),
      };

      _bus.Invoke(new AttachContainerCommand(workspace, floatingWindow));
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
