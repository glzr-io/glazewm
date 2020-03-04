using System.Linq;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.Monitors;
using LarsWM.Domain.Windows;
using LarsWM.Domain.Workspaces.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Workspaces.CommandHandlers
{
    class DisplayWorkspaceHandler : ICommandHandler<DisplayWorkspaceCommand>
    {
        private IBus _bus;
        private MonitorService _monitorService;
        private ContainerService _containerService;

        public DisplayWorkspaceHandler(IBus bus, MonitorService monitorService, ContainerService containerService)
        {
            _bus = bus;
            _monitorService = monitorService;
            _containerService = containerService;
        }

        public dynamic Handle(DisplayWorkspaceCommand command)
        {
            var workspaceToDisplay = command.Workspace;

            var monitor = _monitorService.GetMonitorFromChildContainer(command.Workspace);
            var currentWorkspace = monitor.DisplayedWorkspace;

            // If DisplayedWorkspace is unassigned, there is no need to show/hide windows.
            if (currentWorkspace == null)
            {
                monitor.DisplayedWorkspace = workspaceToDisplay;
                return CommandResponse.Ok;
            }

            if (currentWorkspace == workspaceToDisplay)
                return CommandResponse.Ok;

            var windowsToHide = currentWorkspace.Children
                .SelectMany(container => container.Flatten())
                .OfType<Window>()
                .ToList();

            foreach (var window in windowsToHide)
                window.IsHidden = true;

            var windowsToShow = workspaceToDisplay.Children
                .SelectMany(container => container.Flatten())
                .OfType<Window>()
                .ToList();

            foreach (var window in windowsToShow)
                window.IsHidden = false;

            monitor.DisplayedWorkspace = command.Workspace;

            _containerService.SplitContainersToRedraw.Add(currentWorkspace);
            _containerService.SplitContainersToRedraw.Add(workspaceToDisplay);

            _bus.Invoke(new RedrawContainersCommand());

            return new CommandResponse(true, command.Workspace.Id);
        }
    }
}
