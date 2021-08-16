using LarsWM.Domain.Containers;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;
using static LarsWM.Infrastructure.WindowsApi.WindowsApiService;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class FocusWindowHandler : ICommandHandler<FocusWindowCommand>
  {
    private ContainerService _containerService;

    public FocusWindowHandler(ContainerService containerService)
    {
      _containerService = containerService;
    }

    public dynamic Handle(FocusWindowCommand command)
    {
      var window = command.Window;

      if (window == _containerService.FocusedContainer)
        return CommandResponse.Ok;

      _containerService.FocusedContainer = window;

      // Traverse upwards, creating a focus stack towards the newly focused window.
      // TODO: Not sure whether it's best for the parent containers to point directly
      // to the focused window, or instead point child -> n children -> focused. This would
      // mean Monitor.DisplayedWorkspace could be removed.
      // TODO: Create separate command `CreateFocusStack { TargetContainer }`, which
      // could also be used by `FocusWorkspaceHandler`.
      var parent = window.Parent;
      while (parent != null)
      {
        parent.LastFocusedContainer = window;
        parent = parent.Parent;
      }

      SetForegroundWindow(window.Hwnd);

      return new CommandResponse(true, window.Id);
    }
  }
}
