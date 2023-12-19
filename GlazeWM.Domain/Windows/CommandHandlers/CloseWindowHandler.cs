using System;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class CloseWindowHandler : ICommandHandler<CloseWindowCommand>
  {
    public CommandResponse Handle(CloseWindowCommand command)
    {
      var windowToClose = command.WindowToClose;

      SendNotifyMessage(windowToClose.Handle, SendMessageType.Close, IntPtr.Zero, IntPtr.Zero);

      return CommandResponse.Ok;
    }
  }
}
