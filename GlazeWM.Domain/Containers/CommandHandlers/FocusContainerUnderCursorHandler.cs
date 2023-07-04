using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Containers.CommandHandlers
{
  internal sealed class FocusContainerUnderCursorHandler : ICommandHandler<FocusContainerUnderCursorCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;
    private List<IntPtr> _focusedWindows = new();
    private readonly WindowService _windowService;
    public FocusContainerUnderCursorHandler(Bus bus, ContainerService containerService, WindowService windowService)
    {
      _bus = bus;
      _containerService = containerService;
      _windowService = windowService;
    }

    public CommandResponse Handle(FocusContainerUnderCursorCommand command)
    {
      // Returns window underneath cursor.  This could be a child window or parent.
      var windowHandle = WindowFromPoint(command.TargetPoint);

      // If the mouse is hovering over the currently focused main window or one of it's children, do nothing.
      if (_focusedWindows.Contains(windowHandle))
        return CommandResponse.Ok;

      // If the FocusedWindows list didn't contain the window, this might be a new window being focused.
      var newFocusedWindowsList = new List<IntPtr>
      {
        windowHandle
      };

      // Check if the window is the main window or a child window.
      var parentWindow = GetParent(windowHandle);

      // Walk the window up each parent window until you have the main window.
      while (parentWindow != IntPtr.Zero)
      {
        windowHandle = parentWindow;
        newFocusedWindowsList.Add(windowHandle);
        parentWindow = GetParent(windowHandle);
      }

      // Determine if the same window or a new window is being hovered
      if (_focusedWindows.Contains(newFocusedWindowsList.Last()))
      {
        _focusedWindows = _focusedWindows.Union(newFocusedWindowsList).ToList();
      }
      else
      {
        _focusedWindows = newFocusedWindowsList;
        var foundWindow = _windowService
          .GetWindows()
          .FirstOrDefault(window => window.Handle == windowHandle);

        if (foundWindow is not null)
        {
          // Focus the new window
          SetForegroundWindow(foundWindow.Handle);
          SetFocus(foundWindow.Handle);
        }
      }

      return CommandResponse.Ok;
    }
  }
}
