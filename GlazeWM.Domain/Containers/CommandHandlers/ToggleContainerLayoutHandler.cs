using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class ToggleContainerLayoutHandler : ICommandHandler<ToggleContainerLayoutCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ToggleContainerLayoutHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ToggleContainerLayoutCommand command)
    {
      var container = command.Container;

      if (container is TilingWindow)
      {
        ToggleWindowLayout(container as Window);
      }
      else if (container is Workspace)
      {
        ToggleWorkspaceLayout(container as Workspace);
      }

      // flip between horizontal and vertical modes
      Layout newLayout = (container.Parent as SplitContainer).Layout == Layout.Horizontal ? Layout.Vertical : Layout.Horizontal;

      _bus.Emit(new LayoutChangedEvent(newLayout));

      return CommandResponse.Ok;
    }

    private void ToggleWindowLayout(Window window)
    {
      var parent = window.Parent as SplitContainer;
      var currentLayout = parent.Layout;
      // flip between horizontal and vertical modes
      Layout newLayout = currentLayout == Layout.Horizontal ? Layout.Vertical : Layout.Horizontal;

      // If the window is an only child of a workspace, change layout of the workspace.
      if (!window.HasSiblings() && parent is Workspace)
      {
        ToggleWorkspaceLayout(parent as Workspace);
        return;
      }

      // If the window is an only child and the parent is a normal split container, then flatten
      // the split container.
      if (!window.HasSiblings())
      {
        _bus.Invoke(new FlattenSplitContainerCommand(parent));
        return;
      }

      // Create a new split container to wrap the window.
      var splitContainer = new SplitContainer
      {
        Layout = newLayout,
      };

      // Replace the window with the wrapping split container. The window has to be attached to
      // the split container after the replacement.
      _bus.Invoke(new ReplaceContainerCommand(splitContainer, parent, window.Index));

      // The child window takes up the full size of its parent split container.
      (window as IResizable).SizePercentage = 1;
      _bus.Invoke(new DetachContainerCommand(window));
      _bus.Invoke(new AttachContainerCommand(window, splitContainer));
    }

    private void ToggleWorkspaceLayout(Workspace workspace)
    {
      var currentLayout = workspace.Layout;
      Layout newLayout = currentLayout == Layout.Horizontal ? Layout.Vertical : Layout.Horizontal;

      workspace.Layout = newLayout;

      // Flatten any top-level split containers with the same layout as the workspace. Clone the
      // list since the number of workspace children changes when split containers are flattened.
      foreach (var child in workspace.Children.ToList())
      {
        var childSplitContainer = child as SplitContainer;

        if (childSplitContainer?.Layout != newLayout)
          continue;

        _bus.Invoke(new FlattenSplitContainerCommand(childSplitContainer));
      }

      _containerService.ContainersToRedraw.Add(workspace);
    }
  }
}
