using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Domain.Windows.Events;
using GlazeWM.Infrastructure.Bussing;

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
      var focusedContainer = _containerService.FocusedContainer;
      var focusTarget = window == focusedContainer
        ? WindowService.GetFocusTargetAfterRemoval(window)
        : null;

      if (window is IResizable)
        _bus.Invoke(new DetachAndResizeContainerCommand(window));
      else
        _bus.Invoke(new DetachContainerCommand(window));

      _bus.Emit(new WindowUnmanagedEvent(window.Id, window.Handle));

      if (focusTarget is null)
        return CommandResponse.Ok;

      // The OS automatically switches focus to a different window after closing. If
      // there are focusable windows, then set focus *after* the OS sets focus. This will
      // cause focus to briefly flicker to the OS focus target and then to the WM's focus
      // target.
      _bus.Invoke(new SetFocusedDescendantCommand(focusTarget));
      _containerService.HasPendingFocusSync = true;
      _windowService.UnmanagedOrMinimizedStopwatch.Restart();

      return CommandResponse.Ok;
    }
  }
}
