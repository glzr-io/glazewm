using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers;
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
    private WindowService _windowService;
    private ContainerService _containerService;

    public ToggleFloatingHandler(Bus bus, WorkspaceService workspaceService, WindowService windowService, ContainerService containerService)
    {
      _bus = bus;
      _workspaceService = workspaceService;
      _windowService = windowService;
      _containerService = containerService;
    }

    public CommandResponse Handle(ToggleFloatingCommand command)
    {
      var window = command.Window;

      if (window is FloatingWindow)
        DisableFloating(window as FloatingWindow);

      else
        EnableFloating(window);

      return CommandResponse.Ok;
    }

    private void EnableFloating(Window window)
    {
      // Keep reference to the window's ancestor workspace and focus order index prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(window);
      var focusOrderIndex = window.Parent.ChildFocusOrder.IndexOf(window);

      // Create a floating window and place it in the center of the workspace.
      var floatingWindow = new FloatingWindow(
        window.Hwnd,
        window.OriginalWidth,
        window.OriginalHeight,
        workspace.X + (workspace.Width / 2) - (window.OriginalWidth / 2),
        workspace.Y + (workspace.Height / 2) - (window.OriginalHeight / 2)
      );

      _bus.Invoke(new DetachContainerCommand(window));
      _bus.Invoke(new AttachContainerCommand(workspace, floatingWindow));

      // TODO: This needs to be changed.
      if (focusOrderIndex != -1)
        floatingWindow.Parent.ChildFocusOrder.Insert(focusOrderIndex, floatingWindow);

      _bus.Invoke(new RedrawContainersCommand());
    }

    private void DisableFloating(FloatingWindow floatingWindow)
    {
      // Keep reference to the window's ancestor workspace prior to detaching.
      var workspace = _workspaceService.GetWorkspaceFromChildContainer(floatingWindow);

      // Get the original width and height of the window.
      var originalPlacement = _windowService.GetPlacementOfHandle(floatingWindow.Hwnd).NormalPosition;
      var originalWidth = originalPlacement.Right - originalPlacement.Left;
      var originalHeight = originalPlacement.Bottom - originalPlacement.Top;

      var insertionTarget = workspace.LastFocusedDescendantOfType(typeof(IResizable));

      var tilingWindow = new TilingWindow(floatingWindow.Hwnd, originalWidth, originalHeight);
      _bus.Invoke(new ReplaceContainerCommand(floatingWindow.Parent, floatingWindow.Index, tilingWindow));

      // Descend the tree of the current workspace and insert the created tiling window.
      if (insertionTarget == null)
        _bus.Invoke(new AttachContainerCommand(workspace, tilingWindow));
      else
        _bus.Invoke(
          new MoveContainerWithinTreeCommand(
            tilingWindow,
            insertionTarget.Parent,
            insertionTarget.Index,
            InsertionPosition.AFTER
          )
        );

      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
