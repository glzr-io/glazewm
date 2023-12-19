using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class SetMinimizedHandler : ICommandHandler<SetMinimizedCommand>
  {
    public CommandResponse Handle(SetMinimizedCommand command)
    {
      var window = command.Window;

      ShowWindowAsync(window.Handle, ShowWindowFlags.Minimize);

      return CommandResponse.Ok;
    }
  }
}
