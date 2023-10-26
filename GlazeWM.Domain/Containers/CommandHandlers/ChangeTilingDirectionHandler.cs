using System.Linq;
using GlazeWM.Domain.Common.Enums;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class ChangeTilingDirectionHandler : ICommandHandler<ChangeTilingDirectionCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public ChangeTilingDirectionHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(ChangeTilingDirectionCommand command)
    {
      var container = command.Container;
      var newTilingDirection = command.TilingDirection;

      if (container is TilingWindow)
        ChangeWindowTilingDirection(container as Window, newTilingDirection);
      else if (container is Workspace)
        ChangeWorkspaceTilingDirection(container as Workspace, newTilingDirection);

      _bus.Emit(new TilingDirectionChangedEvent(newTilingDirection));

      return CommandResponse.Ok;
    }

    private void ChangeWindowTilingDirection(
      Window window,
      TilingDirection newTilingDirection)
    {
      var parent = window.Parent as SplitContainer;
      var currentTilingDirection = parent.TilingDirection;

      if (currentTilingDirection == newTilingDirection)
        return;

      // If the window is an only child of a workspace, change tiling direction of the
      // workspace.
      if (!window.HasSiblings() && parent is Workspace)
      {
        ChangeWorkspaceTilingDirection(parent as Workspace, newTilingDirection);
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
        TilingDirection = newTilingDirection,
      };

      // Replace the window with the wrapping split container. The window has to be attached to
      // the split container after the replacement.
      _bus.Invoke(new ReplaceContainerCommand(splitContainer, parent, window.Index));

      // Add the window as a child of the split container and ensure it takes up the full size
      // of its parent split container.
      splitContainer.InsertChild(0, window);
      (window as IResizable).SizePercentage = 1;
    }

    private void ChangeWorkspaceTilingDirection(
      Workspace workspace,
      TilingDirection newTilingDirection)
    {
      var currentTilingDirection = workspace.TilingDirection;

      if (currentTilingDirection == newTilingDirection)
        return;

      workspace.TilingDirection = newTilingDirection;

      // Flatten any top-level split containers with the same tiling direction as the
      // workspace. Clone the list since the number of workspace children changes when
      // split containers are flattened.
      foreach (var child in workspace.Children.ToList())
      {
        var childSplitContainer = child as SplitContainer;

        if (childSplitContainer?.TilingDirection != newTilingDirection)
          continue;

        _bus.Invoke(new FlattenSplitContainerCommand(childSplitContainer));
      }

      _containerService.ContainersToRedraw.Add(workspace);
    }
  }
}
