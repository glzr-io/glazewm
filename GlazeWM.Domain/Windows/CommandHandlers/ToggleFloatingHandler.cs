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

      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);

      // TODO: Consider always detaching the window?
      if (!(window.Parent is Workspace))
        _bus.Invoke(new DetachContainerCommand(window));

      var floatingWindow = new FloatingWindow(window.Hwnd)
      {
        Width = window.OriginalWidth,
        Height = window.OriginalHeight,
        // TODO: Need to place window in the center of the screen.
        X = 400,
        Y = 400,
      };

      _bus.Invoke(new AttachContainerCommand(workspace, floatingWindow));
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
