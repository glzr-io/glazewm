using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class UnmanageWindowHandler : ICommandHandler<UnmanageWindowCommand>
  {
    private readonly Bus _bus;

    public UnmanageWindowHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(UnmanageWindowCommand command)
    {
      var window = command.Window;

      // Keep references to the window's parent and grandparent prior to detaching.
      var parent = window.Parent;
      var grandparent = parent.Parent;

      if (window is IResizable)
        _bus.Invoke(new DetachAndResizeContainerCommand(window));
      else
        _bus.Invoke(new DetachContainerCommand(window));

      // The OS automatically switches focus to a different window after closing. Use `InvokeAsync`
      // to ensure focus gets set to `containerToFocus` *after* the OS sets focus. This will cause
      // focus to briefly flicker to the OS focus target and then to the WM's focus target.
      // TODO: Container to focus should depend on focus mode.
      // TODO: Consider moving this out to `WindowHiddenHandler` and `WindowClosedHandler` after
      // redraw. More likely that it runs after OS focus event.
      var containerToFocus = parent.LastFocusedDescendant ?? grandparent.LastFocusedDescendant;
      _bus.InvokeAsync(new SetNativeFocusCommand(containerToFocus));

      return CommandResponse.Ok;
    }
  }
}
