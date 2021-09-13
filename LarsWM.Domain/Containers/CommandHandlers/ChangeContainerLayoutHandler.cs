using System.Collections.Generic;
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

      // If the window is an only child of a workspace, change layout of the workspace.
      if (!window.HasSiblings() && parent is Workspace)
      {
        ChangeWorkspaceLayout(parent as Workspace, newLayout);
        return;
      }

      // If the window is an only child and the parent is a normal split container, then flatten
      // the split container.
      if (!window.HasSiblings())
      {
        _bus.Invoke(new ReplaceContainerCommand(parent.Parent, parent.Index, new List<Container>() { window }));
        return;
      }

      // Create a new split container to wrap the window.
      var splitContainer = new SplitContainer
      {
        Layout = newLayout,
        ChildFocusOrder = new List<Container> { window },
      };

      // Replace the window with the wrapping split container. The window has to be attached to
      // the split container after the replacement.
      _bus.Invoke(new ReplaceContainerCommand(parent, window.Index, new List<Container>() { splitContainer }));
      _bus.Invoke(new AttachContainerCommand(splitContainer, window));
    }

    private void ChangeWorkspaceLayout(Workspace workspace, Layout newLayout)
    {
      var currentLayout = workspace.Layout;

      if (currentLayout == newLayout)
        return;

      workspace.Layout = newLayout;

      // Flatten any top-level split containers with the same layout as the workspace. Clone
      // the list since the number of workspace children changes when split containers are flattened.
      foreach (var child in workspace.Children.ToList())
      {
        var childSplitContainer = child as SplitContainer;

        if (childSplitContainer == null || childSplitContainer.Layout != newLayout)
          continue;

        _bus.Invoke(new ReplaceContainerCommand(workspace, child.Index, child.Children));
      }

      _containerService.SplitContainersToRedraw.Add(workspace);
    }
  }
}
