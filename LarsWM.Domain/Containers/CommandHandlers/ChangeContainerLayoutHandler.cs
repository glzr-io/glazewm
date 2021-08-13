using System;
using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
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

    public dynamic Handle(ChangeContainerLayoutCommand command)
    {
      var focusedWindow = _windowService.FocusedWindow;
      var parent = focusedWindow.Parent as SplitContainer;

      var newLayout = command.NewLayout;
      var currentLayout = parent.Layout;

      if (currentLayout == newLayout)
        return CommandResponse.Ok;

      var isFocusedOnlyChild = focusedWindow.Siblings.Count() == 0;

      // If the focused window is an only child of a workspace, change layout of workspace.
      if (isFocusedOnlyChild && parent is Workspace)
      {
        parent.Layout = newLayout;
        return CommandResponse.Ok;
      }

      // If the focused container is an only child and the parent is a normal split
      // container, then flatten the split container.
      if (isFocusedOnlyChild)
      {
        var grandparent = parent.Parent;

        var splitContainerIndex = grandparent.Children.IndexOf(parent);

        // Replace the split container with the focused window.
        grandparent.Children[splitContainerIndex] = focusedWindow;
        focusedWindow.Parent = grandparent;
        focusedWindow.SizePercentage = parent.SizePercentage;

        // TODO: Not sure whether redrawing is necessary, will see after fixing detach command.
        _containerService.SplitContainersToRedraw.Add(grandparent as SplitContainer);
        _bus.Invoke(new RedrawContainersCommand());

        return CommandResponse.Ok;
      }

      var splitContainer = new SplitContainer
      {
        Layout = newLayout,
        SizePercentage = focusedWindow.SizePercentage,
        LastFocusedContainer = focusedWindow,
        Parent = parent
      };

      // Get the index of the focused window so that the new split container can be
      // inserted at the same index. Using `AddChild` instead could cause the containers
      // to visually jump around.
      var focusedIndex = parent.Children.IndexOf(focusedWindow);

      _bus.Invoke(new AttachContainerCommand(splitContainer, focusedWindow));

      parent.Children[focusedIndex] = splitContainer;

      _containerService.SplitContainersToRedraw.Add(parent);
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
