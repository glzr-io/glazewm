using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class ChangeContainerLayoutHandler : ICommandHandler<ChangeContainerLayoutCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private WindowService _windowService;

    public ChangeContainerLayoutHandler(Bus bus, ContainerService containerService, WindowService windowService)
    {
      _bus = bus;
      _containerService = containerService;
      _windowService = windowService;
    }

    public CommandResponse Handle(ChangeContainerLayoutCommand command)
    {
      var container = command.Container;
      var newLayout = command.NewLayout;

      if (container is Window)
        ChangeWindowLayout(container as Window, newLayout);

      else if (container is Workspace)
        ChangeWorkspaceLayout(container as Workspace, newLayout);

      return CommandResponse.Ok;
    }

    private void ChangeWindowLayout(Window window, Layout newLayout)
    {
      var parent = window.Parent as SplitContainer;
      var currentLayout = parent.Layout;

      if (currentLayout == newLayout)
        return;

      var isWindowOnlyChild = window.Siblings.Count() == 0;

      // If the window is an only child of a workspace, change layout of the workspace.
      if (isWindowOnlyChild && parent is Workspace)
      {
        ChangeWorkspaceLayout(parent as Workspace, newLayout);
        return;
      }

      // If the window is an only child and the parent is a normal split container, then flatten
      // the split container.
      if (isWindowOnlyChild)
      {
        _bus.Invoke(new ReplaceContainerCommand(parent.Parent, parent.Index, window));
        return;
      }

      // Create a new split container to wrap the window.
      var splitContainer = new SplitContainer
      {
        Layout = newLayout,
      };

      // Replace the window with the wrapping split container. The window has to be attached to
      // the split container after the replacement.
      _bus.Invoke(new ReplaceContainerCommand(parent, window.Index, splitContainer));
      _bus.Invoke(new AttachContainerCommand(splitContainer, window));
    }

    private void ChangeWorkspaceLayout(Workspace workspace, Layout newLayout)
    {
      var currentLayout = workspace.Layout;

      if (currentLayout == newLayout)
        return;

      workspace.Layout = newLayout;

      // TODO: Flatten any top-level split containers with the changed layout of the workspace.

      _bus.Invoke(new RedrawContainersCommand());
    }
  }
}
