using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  class CloseFocusedWindowHandler : ICommandHandler<CloseFocusedWindowCommand>
  {
    private readonly ContainerService _containerService;

    public CloseFocusedWindowHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public CommandResponse Handle(CloseFocusedWindowCommand command)
    {
      // Ignore cases where focused container is not a window or not in foreground.
      if (_containerService.FocusedContainer is not Window focusedWindow || !_containerService.IsFocusSynced)
        return CommandResponse.Ok;

      SendMessage(focusedWindow.Hwnd, SendMessageType.WM_CLOSE, IntPtr.Zero, IntPtr.Zero);

      return CommandResponse.Ok;
    }
  }
}
