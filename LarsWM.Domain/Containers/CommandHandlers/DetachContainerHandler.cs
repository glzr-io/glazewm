using System.Linq;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Workspaces;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Containers.CommandHandlers
{
  class DetachContainerHandler : ICommandHandler<DetachContainerCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private readonly UserConfigService _userConfigService;

    public DetachContainerHandler(Bus bus, ContainerService containerService, UserConfigService userConfigService)
    {
      _bus = bus;
      _containerService = containerService;
      _userConfigService = userConfigService;
    }

    public dynamic Handle(DetachContainerCommand command)
    {
      var parent = command.Parent;
      var childToRemove = command.ChildToRemove;

      parent.RemoveChild(childToRemove);

      AdjustSiblingSizes(parent);

      ChangeFocusedContainer(parent);

      return CommandResponse.Ok;
    }

    private void AdjustSiblingSizes(SplitContainer removedContainerParent)
    {
      // Siblings of the removed child.
      var siblings = removedContainerParent.Children;

      var isEmptySplitContainer = siblings.Count() == 0 && !(removedContainerParent is Workspace);

      double defaultPercent = 1.0 / siblings.Count;

      // If the parent of the removed child is an empty split container, remove
      // the split container as well.
      if (isEmptySplitContainer)
      {
        var grandparent = removedContainerParent.Parent;
        grandparent.RemoveChild(removedContainerParent);

        // TODO: Perhaps create a private method that takes the container with children
        // to adjust that has the SizePercentage and default percent logic. Alternatively
        // create a variable containerToAdjust that is then operated on.
        foreach (var child in grandparent.Children)
          child.SizePercentage = defaultPercent;

        // TODO: Fix issue where grandparent is somehow a split container after vertical ->
        // -> horizontal split.
        _containerService.SplitContainersToRedraw.Add(grandparent as SplitContainer);
        return;
      }

      // TODO: Adjust SizePercentage of children based on their previous SizePercentage.

      foreach (var child in siblings)
        child.SizePercentage = defaultPercent;

      _containerService.SplitContainersToRedraw.Add(removedContainerParent);
    }

    /// <summary>
    /// If the container to remove is the last window in a workspace, then set focus to the
    /// workspace itself. Otherwise, let the OS decide which window to change focus to.
    /// </summary>
    private void ChangeFocusedContainer(SplitContainer removedContainerParent)
    {
      var descendantWindows = removedContainerParent.Flatten().OfType<Window>();

      if (!(removedContainerParent is Workspace) || descendantWindows.Count() > 0)
        return;

      _containerService.FocusedContainer = removedContainerParent;
    }
  }
}
