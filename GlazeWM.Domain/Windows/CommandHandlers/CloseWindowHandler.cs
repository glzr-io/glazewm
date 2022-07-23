using System;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class CloseWindowHandler : ICommandHandler<CloseWindowCommand>
  {
    public CommandResponse Handle(CloseWindowCommand command)
    {
      var windowToClose = command.WindowToClose;

      SendMessage(windowToClose.Hwnd, SendMessageType.WM_CLOSE, IntPtr.Zero, IntPtr.Zero);

      return CommandResponse.Ok;
    }
  }
}
