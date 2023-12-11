using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class ShowAllWindowsHandler : ICommandHandler<ShowAllWindowsCommand>
  {
    private readonly WindowService _windowService;

    public ShowAllWindowsHandler(WindowService windowService)
    {
      _windowService = windowService;
    }

    public CommandResponse Handle(ShowAllWindowsCommand command)
    {
      // Show all windows regardless of whether their workspace is displayed.
      foreach (var window in _windowService.GetWindows())
        ShowWindowAsync(window.Handle, ShowWindowFlags.ShowNoActivate);

      return CommandResponse.Ok;
    }
  }
}
