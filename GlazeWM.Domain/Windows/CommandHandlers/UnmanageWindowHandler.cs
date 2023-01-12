using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using static GlazeWM.Infrastructure.WindowsApi.WindowsApiService;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class UnmanageWindowHandler : ICommandHandler<UnmanageWindowCommand>
  {
    private readonly Bus _bus;
    private readonly ContainerService _containerService;

    public UnmanageWindowHandler(Bus bus, ContainerService containerService)
    {
      _bus = bus;
      _containerService = containerService;
    }

    public CommandResponse Handle(UnmanageWindowCommand command)
    {
      var window = command.Window;

      // Get container to switch focus to after the window has been removed.
      var focusTarget = WindowService.GetFocusTargetAfterRemoval(window);

      if (window is IResizable)
        _bus.Invoke(new DetachAndResizeContainerCommand(window));
      else
        _bus.Invoke(new DetachContainerCommand(window));

      var hasFocusableWindows = WindowService.GetAllWindowHandles().Exists(handle =>
      {
        // TODO: This needs to be changed to match the criteria for the OS to focus a
        // window on close.
        var isVisible = WindowService.IsHandleVisible(handle);
        var canFocus = WindowService.HandleHasWindowExStyle(
          handle,
          WS_EX.WS_EX_NOACTIVATE
        );
        return isVisible && canFocus;
      });

      // The OS automatically switches focus to a different window after closing. If
      // there are focusable windows, then set focus *after* the OS sets focus. This will
      // cause focus to briefly flicker to the OS focus target and then to the WM's focus
      // target.
      if (hasFocusableWindows)
      {
        _containerService.PendingFocusContainer = focusTarget;
        return CommandResponse.Ok;
      }

      _bus.Invoke(new SetNativeFocusCommand(focusTarget));
      return CommandResponse.Ok;
    }
  }
}
