using System.Linq;
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

    public dynamic Handle(ChangeContainerLayoutCommand command)
    {
      var focusedContainer = _containerService.FocusedContainer;
      var parent = focusedContainer.Parent as SplitContainer;

      var newLayout = command.NewLayout;
      var currentLayout = parent.Layout;

      if (currentLayout == newLayout)
        return CommandResponse.Ok;

      var isFocusedOnlyChild = focusedContainer.Siblings.Count() == 0;

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
        _bus.Invoke(new ReplaceContainerCommand(parent.Parent, parent.Index, focusedContainer));
        _bus.Invoke(new RedrawContainersCommand());
        return CommandResponse.Ok;
      }

      // Create a new split container to wrap the focused container.
      var splitContainer = new SplitContainer
      {
        Layout = newLayout,
        LastFocusedContainer = focusedContainer,
      };

      // Replace the focused container with the new split container. The focused window has to be
      // attached to the split container after the replacement.
      _bus.Invoke(new ReplaceContainerCommand(parent, focusedContainer.Index, splitContainer));
      _bus.Invoke(new AttachContainerCommand(splitContainer, focusedContainer));
      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
