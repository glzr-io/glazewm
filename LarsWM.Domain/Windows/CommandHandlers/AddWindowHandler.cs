using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
  class AddWindowHandler : ICommandHandler<AddWindowCommand>
  {
    private Bus _bus;
    private ContainerService _containerService;
    private WindowService _windowService;
    private MonitorService _monitorService;

    public AddWindowHandler(Bus bus, WindowService windowService, MonitorService monitorService, ContainerService containerService)
    {
      _bus = bus;
      _windowService = windowService;
      _monitorService = monitorService;
      _containerService = containerService;
    }

    public dynamic Handle(AddWindowCommand command)
    {
      var window = new Window(command.WindowHandle);

      if (!_windowService.IsWindowManageable(window) || !window.CanLayout)
        return true;

      var focusedContainer = _containerService.FocusedContainer;

      // TODO: If focused container is a workspace, attach it directly rather than to
      // the parent.
      _bus.Invoke(new AttachContainerCommand(focusedContainer.Parent as SplitContainer, window));

      _bus.Invoke(new RedrawContainersCommand());

      // Set focus to newly added window in case it has not been focused automatically.
      _bus.Invoke(new FocusWindowCommand(window));

      return new CommandResponse(true, window.Id);
    }
  }
}
