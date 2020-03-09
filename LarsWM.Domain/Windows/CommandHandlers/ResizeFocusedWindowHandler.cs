using System;
using System.Linq;
using LarsWM.Domain.Common.Enums;
using LarsWM.Domain.Containers;
using LarsWM.Domain.Containers.Commands;
using LarsWM.Domain.UserConfigs;
using LarsWM.Domain.Windows.Commands;
using LarsWM.Infrastructure.Bussing;

namespace LarsWM.Domain.Windows.CommandHandlers
{
    class ResizeFocusedWindowHandler : ICommandHandler<ResizeFocusedWindowCommand>
    {
        private IBus _bus;
        private WindowService _windowService;
        private UserConfigService _userConfigService;
        private ContainerService _containerService;

        public ResizeFocusedWindowHandler(IBus bus, WindowService windowService, UserConfigService userConfigService, ContainerService containerService)
        {
            _bus = bus;
            _windowService = windowService;
            _userConfigService = userConfigService;
            _containerService = containerService;
        }

        public dynamic Handle(ResizeFocusedWindowCommand command)
        {
            var focusedWindow = _windowService.FocusedWindow;
            var parent = focusedWindow.Parent as SplitContainer;
            var siblings = parent.Children.Where(child => child != focusedWindow);

            var resizePercentage = _userConfigService.UserConfig.ResizePercentage;
            var layout = parent.Layout;

            if (layout == Layout.Horizontal && command.Direction == Direction.Right)
            {
                focusedWindow.SizePercentage += resizePercentage;

                foreach (var sibling in siblings)
                    sibling.SizePercentage -= resizePercentage / siblings.Count();
            }

            if (layout == Layout.Horizontal && command.Direction == Direction.Left)
            {
                focusedWindow.SizePercentage -= resizePercentage;

                foreach (var sibling in siblings)
                    sibling.SizePercentage += resizePercentage / siblings.Count();
            }

            _containerService.SplitContainersToRedraw.Add(parent);
            _bus.Invoke(new RedrawContainersCommand());

            return new CommandResponse(true, focusedWindow.Id);
        }
    }
}
