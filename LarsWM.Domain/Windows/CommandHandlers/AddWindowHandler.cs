using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class AddWindowHandler : ICommandHandler<AddWindowCommand>
    {
        private IBus _bus;
        private WindowService _windowService;
        private MonitorService _monitorService;

        public AddWindowHandler(IBus bus, WindowService windowService, MonitorService monitorService)
        {
            _bus = bus;
            _windowService = windowService;
            _monitorService = monitorService;
        }

        public dynamic Handle(AddWindowCommand command)
        {
            var window = new Window(command.WindowHandle);

            if (!_windowService.IsWindowManageable(window) || !window.CanLayout)
                return true;

            // Set initial location values.
            var windowLocation = _windowService.GetLocationOfHandle(command.WindowHandle);
            window.X = windowLocation.Left;
            window.Y = windowLocation.Top;
            window.Width = windowLocation.Right - windowLocation.Left;
            window.Height = windowLocation.Bottom - windowLocation.Top;

            var focusedWindow = _windowService.FocusedWindow;

            _bus.Invoke(new AttachContainerCommand(focusedWindow.Parent as SplitContainer, window));

            _bus.Invoke(new RedrawContainersCommand());

            return new CommandResponse(true, window.Id);
        }
    }
}
