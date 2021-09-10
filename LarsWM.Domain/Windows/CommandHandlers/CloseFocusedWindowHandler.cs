using System;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class CloseFocusedWindowHandler : ICommandHandler<CloseFocusedWindowCommand>
  {
    private ContainerService _containerService;

    public CloseFocusedWindowHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public dynamic Handle(CloseFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;
      var foregroundWindow = GetForegroundWindow();

      // Ignore cases where focused container is not a window or not in foreground.
      if (focusedWindow == null || foregroundWindow != focusedWindow.Hwnd)
        return CommandResponse.Ok;

      SendMessage(focusedWindow.Hwnd, SendMessageType.WM_CLOSE, IntPtr.Zero, IntPtr.Zero);

      return CommandResponse.Ok;
    }
  }
}
