using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class ToggleFloatingHandler : ICommandHandler<ToggleFloatingCommand>
  {
    private readonly Bus _bus;

    public ToggleFloatingHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(ToggleFloatingCommand command)
    {
      var window = command.Window;

      if (window is FloatingWindow)
        UnsetFloating(window as FloatingWindow);
      else
        _bus.Invoke(new SetFloatingCommand(window));

      return CommandResponse.Ok;
    }

    private void UnsetFloating(FloatingWindow floatingWindow)
    {
      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(floatingWindow);

      var insertionTarget = workspace.LastFocusedDescendantOfType(typeof(IResizable));

      var tilingWindow = new TilingWindow(
        floatingWindow.Handle,
        floatingWindow.FloatingPlacement,
        floatingWindow.BorderDelta,
        0
      );

      _bus.Invoke(
        new ReplaceContainerCommand(
          tilingWindow,
          floatingWindow.Parent,
          floatingWindow.Index
        )
      );

      // Insert the created tiling window after the last focused descendant of the workspace.
      if (insertionTarget == null)
        _bus.Invoke(new MoveContainerWithinTreeCommand(tilingWindow, workspace, 0, true));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            tilingWindow,
            insertionTarget.Parent,
            insertionTarget.Index + 1,
            true
          )
        );

      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
