using System;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class SetNativeFocusHandler : ICommandHandler<SetNativeFocusCommand>
  {
    private readonly Bus _bus;

    public SetNativeFocusHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(SetNativeFocusCommand command)
    {
      var containerToFocus = command.ContainerToFocus;

      var handleToFocus = containerToFocus switch
      {
        Window window => window.Handle,
        Workspace => GetDesktopHandle(),
        _ => throw new Exception("Invalid container type to focus. This is a bug."),
      };

      // Set focus to the given window handle. If the container is a normal window, then this
      // will trigger `EVENT_SYSTEM_FOREGROUND` window event and its handler. This, in turn,
      // calls `SetFocusedDescendantCommand`.
      KeybdEvent(0, 0, 0, 0);
      SetForegroundWindow(handleToFocus);

      // Setting focus to the desktop window does not emit `EVENT_SYSTEM_FOREGROUND` window event,
      // so `SetFocusedDescendantCommand` has to be manually called.
      if (containerToFocus is Workspace)
      {
        _bus.Invoke(new SetFocusedDescendantCommand(containerToFocus));
        _bus.Emit(new FocusChangedEvent(containerToFocus));
      }

      _bus.InvokeAsync(new CenterCursorOnRectCommand(containerToFocus.ToRect()));

      return CommandResponse.Ok;
    }

    private static IntPtr GetDesktopHandle()
    {
      return WindowService.GetAllWindowHandles().Find(handle =>
      {
        var className = WindowService.GetClassNameOfHandle(handle);
        var process = WindowService.GetProcessOfHandle(handle);
        return className == "Progman" && process.ProcessName == "explorer";
      });
    }
  }
}
