using System;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class CloseFocusedWindowHandler : ICommandHandler<CloseFocusedWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;

    public CloseFocusedWindowHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public dynamic Handle(CloseFocusedWindowCommand command)
    {
      var focusedWindow = _containerService.FocusedContainer as Window;

      if (focusedWindow == null)
        return CommandResponse.Ok;

      SendMessage(focusedWindow.Hwnd, SendMessageType.WM_CLOSE, IntPtr.Zero, IntPtr.Zero);

      return CommandResponse.Ok;
    }
  }
}
