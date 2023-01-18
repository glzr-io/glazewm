using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class SetFloatingHandler : ICommandHandler<SetFloatingCommand>
  {
    private readonly Bus _bus;

    public SetFloatingHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(SetFloatingCommand command)
    {
      var window = command.Window;

      if (window is FloatingWindow)
        return CommandResponse.Ok;

      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);

      if (window is IResizable)
        _bus.Invoke(new MoveContainerWithinTreeCommand(window, workspace, true));
      else
        _bus.Invoke(new MoveContainerWithinTreeCommand(window, workspace, false));

      // Create a floating window and place it in the center of the workspace.
      var floatingWindow = new FloatingWindow(
        window.Handle,
        window.FloatingPlacement,
        window.BorderDelta
      );

      _bus.Invoke(new ReplaceContainerCommand(floatingWindow, window.Parent, window.Index));
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
