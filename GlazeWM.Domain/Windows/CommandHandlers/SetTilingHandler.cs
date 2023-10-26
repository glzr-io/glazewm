using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class SetTilingHandler : ICommandHandler<SetTilingCommand>
  {
    private readonly Bus _bus;

    public SetTilingHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(SetTilingCommand command)
    {
      var window = command.Window;

      if (window is TilingWindow)
        return CommandResponse.Ok;

      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = WorkspaceService.GetWorkspaceFromChildContainer(window);

      var insertionTarget = workspace.LastFocusedDescendantOfType<IResizable>();

      var tilingWindow = new TilingWindow(
        window.Handle,
        window.FloatingPlacement,
        window.BorderDelta,
        0
      )
      {
        Id = window.Id
      };

      // Replace the original window with the created tiling window.
      _bus.Invoke(new ReplaceContainerCommand(tilingWindow, window.Parent, window.Index));

      // Insert the created tiling window after the last focused descendant of the workspace.
      if (insertionTarget is null)
        _bus.Invoke(new MoveContainerWithinTreeCommand(tilingWindow, workspace, 0));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            tilingWindow,
            insertionTarget.Parent,
            insertionTarget.Index + 1
          )
        );

      return CommandResponse.Ok;
    }
  }
}
