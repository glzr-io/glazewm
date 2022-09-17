using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Containers.Commands;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal class IgnoreWindowHandler : ICommandHandler<IgnoreWindowCommand>
  {
    private readonly Bus _bus;
    private readonly WindowService _windowService;

    public IgnoreWindowHandler(Bus bus, WindowService windowService)
    {
      _bus = bus;
      _windowService = windowService;
    }

    public CommandResponse Handle(IgnoreWindowCommand command)
    {
      var window = command.WindowToIgnore;

      // Store handle to ignored window.
      _windowService.IgnoredHandles.Add(window.Hwnd);

      if (window is IResizable)
        _bus.Invoke(new DetachAndResizeContainerCommand(window));
      else
        _bus.Invoke(new DetachContainerCommand(window));

      _bus.Invoke(new RedrawContainersCommand());

      return CommandResponse.Ok;
    }
  }
}
