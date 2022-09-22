using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal class SetNativeFocusHandler : ICommandHandler<SetNativeFocusCommand>
  {
    private readonly Bus _bus;

    public SetNativeFocusHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(SetNativeFocusCommand command)
    {
      var containerToFocus = command.ContainerToFocus;

      if (containerToFocus is Window)
      {
        // Set as foreground window if it's not already set. This will trigger `EVENT_SYSTEM_FOREGROUND`
        // window event and its handler. This, in turn, calls `SetFocusedDescendant`.
        KeybdEvent(0, 0, 0, 0);
        SetForegroundWindow((containerToFocus as Window).Handle);
      }
      else if (containerToFocus is Workspace)
      {
        // Setting focus to the desktop window does not emit `EVENT_SYSTEM_FOREGROUND` window event, so
        // `SetFocusedDescendant` has to be manually called.
        KeybdEvent(0, 0, 0, 0);
        SetForegroundWindow(GetDesktopWindow());
        _bus.Invoke(new SetFocusedDescendantCommand(containerToFocus));
      }
      else
        throw new Exception("Invalid container type to focus. This is a bug.");

      return CommandResponse.Ok;
    }
  }
}
