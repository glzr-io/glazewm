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

    public RemoveWindowHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(RemoveWindowCommand command)
    {
      var window = command.Window;

      // Keep references to the window's original parent and grandparent prior to detaching.
      var parent = window.Parent;
      var grandparent = parent.Parent;

      _bus.Invoke(new DetachContainerCommand(window.Parent as SplitContainer, window));

      // Search for a new container to set focus to.
      var containerToFocus = parent.LastFocusedDescendant ?? grandparent.LastFocusedDescendant;

      // Note that the hook that fires when a window closes is actually called AFTER the OS has
      // automatically switched focus to a new window. So therefore, changing focus here will
      // cause focus to briefly flicker to and from what the OS wants to focus on.
      if (containerToFocus is Window)
        _bus.Invoke(new FocusWindowCommand(containerToFocus as Window));
      else if (containerToFocus is Workspace)
        _bus.Invoke(new FocusWorkspaceCommand((containerToFocus as Workspace).Name));

      return CommandResponse.Ok;
    }
  }
}
