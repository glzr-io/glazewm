using System;
using System.Linq;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Domain.Workspaces;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class RemoveWindowHandler : ICommandHandler<RemoveWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public RemoveWindowHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(RemoveWindowCommand command)
    {
      var window = command.Window;

      // Keep references to the window's parent and grandparent prior to detaching.
      var parent = window.Parent;
      var grandparent = parent.Parent;

      _bus.Invoke(new DetachContainerCommand(window.Parent as SplitContainer, window));

      // Get container to switch focus to after the window has been removed. The OS automatically
      // switches focus to a different window after closing, so by setting `PendingFocusContainer`
      // this behavior is overridden.
      var containerToFocus = parent.LastFocusedDescendant ?? grandparent.LastFocusedDescendant;
      _containerService.PendingFocusContainer = containerToFocus;

      return CommandResponse.Ok;
    }
  }
}
