using System;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class HideTitleBarHandler : ICommandHandler<HideTitleBarCommand>
  {
    public CommandResponse Handle(HideTitleBarCommand command)
    {
      var target = command.TargetWindow;
      var style = GetWindowLongPtr(target.Handle, GWLSTYLE);
      SetWindowLongPtr(target.Handle, GWLSTYLE, new IntPtr(style.ToInt32() & ~WS_CAPTION));
      SetWindowPos(target.Handle, IntPtr.Zero, 0, 0, 0, 0,
        WindowsApiService.SetWindowPosFlags.NoMove |
        WindowsApiService.SetWindowPosFlags.NoSize |
        WindowsApiService.SetWindowPosFlags.FrameChanged);

      return CommandResponse.Ok;
    }
  }
}
